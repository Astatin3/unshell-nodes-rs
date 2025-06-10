use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Debug, Serialize, Deserialize)]
pub enum Packets {
    GetConnections,
    UpdateConnections(Vec<String>),

    GetRoutes,
    UpdateRoutes(Vec<String>),

    OnClientConnect { id: String, route: Vec<String> },
    OnClientDisconnect { id: String },

    Error(PacketError),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PacketError {
    UnsupportedType,
}

impl Packets {
    pub fn encode(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(self)?)
    }
    pub fn decode(string: &str) -> Result<Self, Error> {
        Ok(serde_json::from_str::<Self>(string)?)
    }
}
