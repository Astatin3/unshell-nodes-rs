use std::net::SocketAddr;

use bincode::{Decode, Encode};

use crate::layers::LayerConfig;

#[derive(Encode, Decode, Debug, Clone)]
pub struct ConnectionConfig {
    pub socket: SocketAddr,
    pub layers: Vec<LayerConfig>,
}
