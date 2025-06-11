use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Packets {
    UpdateRoutes(String, Vec<String>),
    Connect(String),
    Disconnect(String),
    Data { source: String, data: String },
}

impl Packets {
    pub fn encode(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(self)?)
    }
    pub fn decode(string: &str) -> Result<Self, Error> {
        Ok(serde_json::from_str::<Self>(string)?)
    }
}
