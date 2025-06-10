use std::net::SocketAddr;

use crate::Error;

// This is the data transmission type
pub trait Connection: Send + Sync {
    fn get_info(&self) -> String;
    fn is_alive(&self) -> bool;

    fn read(&mut self) -> Result<String, Error>;
    fn write(&mut self, data: &str) -> Result<(), Error>;

    fn try_clone(&self) -> Result<Box<dyn Connection + Send + Sync>, Error>;
}

// Trait for protocol layers that can be initialized
pub trait ProtocolLayer: Connection {
    fn new(inner: Box<dyn Connection>) -> Result<Self, Error>
    where
        Self: Sized;
    fn initialize_client(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn initialize_server(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

// impl Sized for dyn Connection {}

// pub trait AsyncConnection<C>
// where
//     C: Connection,
// {
//     fn as_async<T: Serialize + DeserializeOwned + Send + 'static>(
//         connection: C,
//     ) -> (Sender<T>, Receiver<T>);
// }

pub trait ServerTrait<C: Connection> {
    fn get_info(&self) -> String;
    fn accept(&self) -> Result<C, Error>;
    fn bind(address: &SocketAddr) -> Result<Self, Error>
    where
        Self: Sized;
}

pub trait ClientTrait<C: Connection + Sized> {
    fn connect(address: &SocketAddr) -> Result<C, Error>;
}
