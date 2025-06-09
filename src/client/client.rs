use std::{
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use crossbeam_channel::{Receiver, Sender};
use unshell_rs_lib::{
    connection::{C2Packet, Parameter, Parameters},
    networkers::{AsyncConnection, ClientTrait, TCPClient, TCPConnection},
};

pub struct UnshellClient {
    #[allow(dead_code)]
    addr: SocketAddr,
    tx: Sender<C2Packet>,
    pub rx: Receiver<C2Packet>,
    parameters: Arc<Mutex<Parameters>>,
}

impl UnshellClient {
    pub fn new(addr: SocketAddr) -> Result<Self, Box<dyn Error>> {
        let client = TCPClient::connect(&addr)?;

        let (tx, rx) = TCPConnection::as_async(client);

        let parameters = Arc::new(Mutex::new(Parameters::new()));

        Ok(Self {
            addr,
            tx,
            rx,
            parameters,
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
