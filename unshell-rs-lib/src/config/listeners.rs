use std::{
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::{
    config::layers::LayerConfig,
    networkers::{ServerTrait, TCPConnection, TCPServer},
};

#[derive(Serialize, Deserialize, Clone)]
pub enum ListenerConfig {
    Tcp {
        enabled: bool,
        name: String,
        addr: SocketAddr,
        layers: Vec<LayerConfig>,

        #[serde(skip)]
        connections: Option<Arc<Mutex<Vec<TCPConnection>>>>,
    },
}

impl ListenerConfig {
    pub fn start(self) -> Result<(), Box<dyn Error>> {
        match self {
            ListenerConfig::Tcp {
                mut enabled,
                addr,
                layers,
                mut connections,
                ..
            } => {
                let server = TCPServer::bind(&addr)?;

                enabled = true;

                // connections = Some(run_listener(server));
            }
        }

        Ok(())
    }
}
