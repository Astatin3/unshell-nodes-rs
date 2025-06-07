use serde_json::Result;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use unshell_rs_lib::connection::ErrorPacket;

pub type Parameters = HashMap<String, Parameter>;

#[derive(Debug, Serialize, Deserialize)]
pub enum GuiPacket {
    GetParameter(String),
    AckGetParameter(String, Option<Parameter>),
    ParameterUpate(String, Parameter),

    SetParameter(String, Parameter),
    AckSetParameter(bool),

    SetAllParameters(Parameters),

    Error(ErrorPacket),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Parameter {
    Test1,
    CurrentTab(i32),
}

impl GuiPacket {
    pub fn encode(&self) -> Result<String> {
        serde_json::to_string(self)
    }

    pub fn decode(string: &str) -> Result<Self> {
        serde_json::from_str::<Self>(string)
    }
}
