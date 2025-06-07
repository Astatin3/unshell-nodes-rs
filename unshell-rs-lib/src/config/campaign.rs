use serde::{Deserialize, Serialize};

use crate::config::listeners::ListenerConfig;

#[derive(Serialize, Deserialize, Clone)]
pub struct CampaignConfig {
    pub name: String,
    pub listeners: Vec<ListenerConfig>,
}
