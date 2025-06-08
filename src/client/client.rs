use std::{
    error::Error,
    mem,
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crossbeam_channel::{Receiver, Sender};
use unshell_rs_lib::{
    connection::{C2Packet, Parameter, Parameters},
    networkers::{AsyncConnection, ClientTrait, TCPClient, TCPConnection},
};

pub struct UnshellClient {
    addr: SocketAddr,
    tx: Sender<C2Packet>,
    rx: Receiver<C2Packet>,
    parameters: Arc<Mutex<Parameters>>,
}

impl UnshellClient {
    pub fn new(addr: SocketAddr) -> Result<Self, Box<dyn Error>> {
        let client = TCPClient::connect(&addr)?;

        let (tx, rx) = TCPConnection::as_async(client);

        // mpsc

        // let poll = Poll::new()?;

        // let events = Events::with_capacity(128);

        // const SERVER: Token = Token(0);
        // poll.registry()
        //     .register(&mut listener, SERVER, Interest::READABLE)?;

        // let client = Arc::new(Mutex::new());
        // let outgoing_packets = Arc::new(Mutex::new(Vec::<GuiPacket>::new()));
        let parameters = Arc::new(Mutex::new(Parameters::new()));

        // let tx_client = Arc::clone(&client);
        // let tx_packets = Arc::clone(&outgoing_packets);

        // // Recieve thread
        // thread::spawn(move || {
        //     loop {
        //         info!("Lock 2");
        //         let mut packets_lock = tx_packets.lock().unwrap();
        //         info!("Lock 2");
        //         if !packets_lock.is_empty() {
        //             info!("Lock 3");
        //             if let Ok(packet) = packets_lock.pop().unwrap().encode() {
        //                 info!("Lock 3");
        //                 info!("Lock 4");
        //                 let mut client_lock = tx_client.lock().unwrap();
        //                 info!("Lock 4");
        //                 info!("Wrote {}", packet.as_str());
        //                 match client_lock.write(packet.as_str()) {
        //                     Err(e) => {
        //                         error!("Failed to send packet: {:?}", e);
        //                     }
        //                     _ => {}
        //                 };
        //             }
        //         }
        //         std::mem::drop(packets_lock);

        //         thread::sleep(Duration::from_millis(10));
        //     }
        // });

        // let rx_client = Arc::clone(&client);
        // let rx_params = Arc::clone(&parameters);
        // thread::spawn(move || {
        //     loop {
        //         info!("Lock 5");
        //         let mut client = rx_client.lock().unwrap();
        //         info!("Lock 5");
        //         if !client.is_alive() {
        //             error!("Disconnected from {}!", client.get_info());
        //         }
        //         if let Ok(data) = client.read() {
        //             info!("Got {}", data);
        //             if let Ok(packet) = GuiPacket::decode(data.as_str()) {
        //                 match packet {
        //                     GuiPacket::ParameterUpate(name, parameter) => {
        //                         rx_params.lock().unwrap().insert(name, parameter);
        //                     }
        //                     GuiPacket::Error(error_packet) => {
        //                         error!("Got error: {}", print_type_of(&error_packet))
        //                     }
        //                     GuiPacket::SetAllParameters(parameters) => {
        //                         let mut params_lock = rx_params.lock().unwrap();
        //                         params_lock.clear();
        //                         params_lock.extend(parameters);
        //                     }
        //                     _ => {
        //                         error!("Unsupported packet: {}", data)
        //                     }
        //                 }
        //             }
        //         }
        //         // std::mem::drop(client);

        //         thread::sleep(Duration::from_millis(10));
        //     }
        // });

        Ok(Self {
            addr,

            tx,
            rx,

            // client,
            parameters,
            // outgoing_packets,
        })
    }

    pub fn set_parameter(&mut self, key: String, param: Parameter) {
        let mut params_lock = self.parameters.lock().unwrap();
        params_lock.insert(key.clone(), param.clone());
        self.tx.send(C2Packet::SetParameter(key, param)).unwrap();
    }

    pub fn get_parameter(&self, key: &str) -> Option<Parameter> {
        self.parameters.lock().unwrap().get(key).cloned()
    }
}

fn print_type_of<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}
