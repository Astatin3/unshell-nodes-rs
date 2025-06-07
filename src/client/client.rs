use std::{
    error::Error,
    mem,
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use unshell_rs_lib::networkers::{ClientTrait, Connection, TCPClient, TCPConnection};

use crate::packets::{GuiPacket, Parameter, Parameters};

pub struct UnshellClient {
    addr: SocketAddr,
    client: Arc<Mutex<TCPConnection>>,

    parameters: Arc<Mutex<Parameters>>,
    outgoing_packets: Arc<Mutex<Vec<GuiPacket>>>,
}

impl UnshellClient {
    pub fn new(addr: SocketAddr) -> Result<Self, Box<dyn Error>> {
        let client = Arc::new(Mutex::new(TCPClient::connect(&addr)?));
        let outgoing_packets = Arc::new(Mutex::new(Vec::<GuiPacket>::new()));
        let parameters = Arc::new(Mutex::new(Parameters::new()));

        let tx_client = Arc::clone(&client);
        let tx_packets = Arc::clone(&outgoing_packets);

        // Recieve thread
        thread::spawn(move || {
            loop {
                info!("Lock 2");
                let mut packets_lock = tx_packets.lock().unwrap();
                info!("Lock 2");
                if !packets_lock.is_empty() {
                    info!("Lock 3");
                    if let Ok(packet) = packets_lock.pop().unwrap().encode() {
                        info!("Lock 3");
                        let mut client_lock = tx_client.lock().unwrap();
                        info!("Wrote {}", packet.as_str());
                        match client_lock.write(packet.as_str()) {
                            Err(e) => {
                                error!("Failed to send packet: {:?}", e);
                            }
                            _ => {}
                        };
                    }
                }
                std::mem::drop(packets_lock);

                thread::sleep(Duration::from_millis(10));
            }
        });

        let rx_client = Arc::clone(&client);
        let rx_params = Arc::clone(&parameters);
        thread::spawn(move || {
            loop {
                info!("Lock 4");
                let mut client = rx_client.lock().unwrap();
                info!("Lock 4");
                if !client.is_alive() {
                    error!("Disconnected from {}!", client.get_info());
                }
                if let Ok(data) = client.read() {
                    info!("Got {}", data);
                    if let Ok(packet) = GuiPacket::decode(data.as_str()) {
                        match packet {
                            GuiPacket::ParameterUpate(name, parameter) => {
                                rx_params.lock().unwrap().insert(name, parameter);
                            }
                            GuiPacket::Error(error_packet) => {
                                error!("Got error: {}", print_type_of(&error_packet))
                            }
                            GuiPacket::SetAllParameters(parameters) => {
                                let mut params_lock = rx_params.lock().unwrap();
                                params_lock.clear();
                                params_lock.extend(parameters);
                            }
                            _ => {
                                error!("Unsupported packet: {}", data)
                            }
                        }
                    }
                }
                // std::mem::drop(client);

                thread::sleep(Duration::from_millis(10));
            }
        });

        Ok(Self {
            addr,
            client,
            parameters,
            outgoing_packets,
        })
    }

    pub fn set_parameter(&mut self, key: String, param: Parameter) {
        self.parameters
            .lock()
            .unwrap()
            .insert(key.clone(), param.clone());
        self.outgoing_packets
            .lock()
            .unwrap()
            .push(GuiPacket::SetParameter(key, param));
    }

    pub fn get_parameter(&self, key: &str) -> Option<Parameter> {
        self.parameters.lock().unwrap().get(key).cloned()
    }
}

fn print_type_of<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}
