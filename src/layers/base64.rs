use crate::layers::Layer;
use base64;

#[derive(Default)]
pub struct Base64;

impl Layer for Base64 {
    fn encode(&mut self, data: &[u8]) -> Vec<u8> {
        #[allow(deprecated)]
        base64::encode(str::from_utf8(data).unwrap()).into_bytes()
    }

    fn decode(&mut self, data: &[u8]) -> Vec<u8> {
        #[allow(deprecated)]
        base64::decode(str::from_utf8(data).unwrap()).unwrap()
    }
}
