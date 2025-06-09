use crate::{
    Error,
    networkers::{Connection, ProtocolLayer},
};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};

pub struct Base64Layer<C: Connection> {
    inner: C,
}

impl<C: Connection> Connection for Base64Layer<C> {
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
        info!("Bsae");

        self.inner.write(&general_purpose::STANDARD.encode(data))?;

        Ok(())
    }
}

impl<C: Connection> ProtocolLayer<C> for Base64Layer<C> {
    fn new(inner: C) -> Result<Self, Error> {
        Ok(Base64Layer { inner })
    }
}
