use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::layers::LayerConfig;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionConfig {
    pub socket: SocketAddr,
    pub layers: Vec<LayerConfig>,
}
