use crate::{
    Error,
    networkers::{Connection, ProtocolLayer},
};
use base64::{Engine as _, engine::general_purpose};

pub struct Base64Layer {
    inner: Box<dyn Connection>,
}

impl Connection for Base64Layer {
    fn get_info(&self) -> String {
        format!("b64->{}", self.inner.get_info())
    }

    fn is_alive(&self) -> bool {
        self.inner.is_alive()
    }

    fn read(&mut self) -> Result<String, Error> {
        Ok(str::from_utf8(
            &general_purpose::STANDARD
                .decode(&self.inner.read()?)
                .unwrap(),
        )
        .unwrap()
        .to_string())
    }

    fn write(&mut self, data: &str) -> Result<(), Error> {
        self.inner.write(&general_purpose::STANDARD.encode(data))
    }

    fn try_clone(&self) -> Result<Box<dyn Connection + Send + Sync>, Error> {
        Ok(Box::new(Self {
            inner: self.inner.try_clone()?,
        }))
    }
}

impl ProtocolLayer for Base64Layer {
    fn new(inner: Box<dyn Connection>) -> Result<Self, Error> {
        Ok(Base64Layer { inner })
    }
}
