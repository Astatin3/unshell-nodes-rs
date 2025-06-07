use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Result;

use crate::config::campaign::CampaignConfig;

#[derive(Serialize, Deserialize, Debug)]
pub enum C2Packet {
    GetClients,
    AckGetClients,

    RequestCampaign,
    AckRequestCampaign(CampaignConfig),

    SetCampaign(CampaignConfig),
    AckSetCampaign,

    Error(ErrorPacket),

    Sysinfo { hostname: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ErrorPacket {
    UnsupportedRequestError,
}

impl fmt::Debug for CampaignConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CampaignConfig")
    }
}

impl C2Packet {
    pub fn encode(&self) -> Result<String> {
        serde_json::to_string(self)
    }

    pub fn decode(string: &str) -> Result<Self> {
        serde_json::from_str::<Self>(string)
    }
}
