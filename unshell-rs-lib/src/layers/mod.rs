pub trait Layer: Serialize + Deserialize<'static> + Sized {
    fn encode(&mut self, data: &[u8]) -> Vec<u8>;
    fn decode(&mut self, data: &[u8]) -> Vec<u8>;
}

pub mod base64;

pub use base64::Base64;
use serde::{Deserialize, Serialize};
