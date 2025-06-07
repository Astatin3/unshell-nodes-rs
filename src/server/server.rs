use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::Read,
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};

use unshell_rs_lib::{
    config::campaign::CampaignConfig,
    connection::ErrorPacket,
    networkers::{Connection, ServerTrait, TCPConnection, TCPServer, run_listener_state},
};

use crate::{
    packets::{GuiPacket, Parameters},
    server::{DEFAULT_CAMPAIGN, DEFAULT_USERS, User, config::DEFAULT_PARAMETERS},
};

#[derive(Serialize, Deserialize)]
pub struct UnshellServerConfig {
    campaign: CampaignConfig,
    parameters: Parameters,
    users: Vec<User>,

    #[serde(skip)]
    client_count: usize,
    #[serde(skip)]
    broadcast_flag: HashMap<usize, Option<String>>,
}

pub struct UnshellServer {
    config: Arc<Mutex<UnshellServerConfig>>,
}

impl UnshellServer {
    pub fn from_filepath(filepath: &str) -> Self {
        (|| -> Result<Self, Box<dyn Error>> {
            let mut file = File::open(filepath.to_string())?;

            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            let config = serde_json::from_str::<UnshellServerConfig>(contents.as_str())?;

            info!("Loaded server config from {}", filepath);

            Ok(Self {
                config: Arc::new(Mutex::new(config)),
            })
        })()
        .unwrap_or({
            warn!("Loaded default server config");
            Self {
                config: Arc::new(Mutex::new(UnshellServerConfig {
                    campaign: DEFAULT_CAMPAIGN.clone(),
                    users: DEFAULT_USERS.clone(),
                    parameters: DEFAULT_PARAMETERS.clone(),

                    client_count: 0,
                    broadcast_flag: HashMap::new(),
                })),
            }
        })
    }

    pub fn run(&mut self, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
        let on_connect = |connection: TCPConnection,
                          config_clone: Arc<Mutex<UnshellServerConfig>>| {
            // Recieve loop
            thread::spawn(move || {
                let config = Arc::clone(&config_clone);

                let mut connection = connection;

                let send = |c: &mut TCPConnection, packet: GuiPacket| {
                    if let Ok(packet) = packet.encode() {
                        info!("Send {}", packet);
                        c.write(packet.as_str()).unwrap();
                    }
                };

                let mut config_lock = config.lock().unwrap();
                let client_id = config_lock.client_count.clone();
                config_lock.client_count += 1;
                send(
                    &mut connection,
                    GuiPacket::SetAllParameters(config_lock.parameters.clone()),
                );
                std::mem::drop(config_lock);

                loop {
                    if !connection.is_alive() {
                        warn!("Client {} disconnected!", connection.get_info());
                        break;
                    }
                    if let Ok(data) = connection.read() {
                        if let Ok(packet) = GuiPacket::decode(data.as_str()) {
                            match packet {
                                GuiPacket::GetParameter(param) => send(
                                    &mut connection,
                                    GuiPacket::AckGetParameter(param.clone(), {
                                        let config_lock = config.lock().unwrap();
                                        let result = config_lock.parameters.get(&param);
                                        result.cloned()
                                    }),
                                ),
                                GuiPacket::SetParameter(name, param) => send(
                                    &mut connection,
                                    GuiPacket::AckSetParameter({
                                        let mut config_lock = config.lock().unwrap();
                                        config_lock.parameters.insert(name.clone(), param);
                                        config_lock.broadcast_flag.insert(client_id, Some(name));

                                        true
                                    }),
                                ),
                                _ => send(
                                    &mut connection,
                                    GuiPacket::Error(ErrorPacket::UnsupportedRequestError),
                                ),
                            }
                        }
                    }

                    let mut config_lock = config.lock().unwrap();
                    if let Some(Some(key)) = config_lock.broadcast_flag.get(&client_id) {
                        send(
                            &mut connection,
                            GuiPacket::ParameterUpate(
                                key.clone(),
                                config_lock.parameters.get(key).unwrap().clone(),
                            ),
                        );
                        config_lock.broadcast_flag.insert(client_id, None);
                    }

                    thread::sleep(Duration::from_millis(10));
                }
            });
        };

        let config_clone = Arc::clone(&self.config);
        run_listener_state(TCPServer::bind(&addr)?, on_connect, config_clone);

        Ok(())
    }
}
