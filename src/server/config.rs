use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use unshell_rs_lib::config::campaign::CampaignConfig;

use std::collections::HashMap;

use crate::packets::Parameters;

lazy_static! {
    pub static ref DEFAULT_CAMPAIGN: CampaignConfig = CampaignConfig {
        name: "Default Campaign".to_string(),
        listeners: Vec::new(),
    };
    pub static ref DEFAULT_USERS: Vec<User> = vec![User {
        name: "User".into(),
        key: "CHANGEME".to_string(),
    }];
    pub static ref DEFAULT_PARAMETERS: Parameters = {
        let p = Parameters::new();

        p
    };
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub name: String,
    pub key: String,
}
