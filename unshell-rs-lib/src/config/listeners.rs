use serde::{Deserialize, Serialize};

use crate::config::layers::LayerConfig;

#[derive(Debug, Serialize, Deserialize)]
pub enum ListenerConfig {
    Tcp {
        enabled: bool,
        name: String,
        remote_host: String,
        port: u16,
        layers: Vec<LayerConfig>,
    },
}
