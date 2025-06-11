use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LayerConfig {
    Base64,
    Handshake,
}

mod base64;
mod builder;
mod handshake;

pub use base64::Base64Layer;
pub use handshake::HandshakeLayer;

pub use builder::build_client;
pub use builder::create_server_builder;
