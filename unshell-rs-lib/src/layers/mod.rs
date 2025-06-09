pub enum LayerConfig {
    Base64,
    Handshake,
}

pub mod base64;
mod builder;
pub mod handshake;

pub use base64::Base64Layer;
pub use handshake::HandshakeLayer;

pub use builder::build_client;
pub use builder::create_server_builder;
