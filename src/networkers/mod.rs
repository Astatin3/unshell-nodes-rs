/// This is the lowset-level data transmission type

pub trait Connection: Send + Sync {
    type Error: std::fmt::Debug;

    fn read(&mut self) -> Result<String, Self::Error>;
    fn write(&mut self, data: &str) -> Result<(), Self::Error>;
}

pub trait ServerTrait<C: Connection> {
    type Error: std::fmt::Debug;

    fn accept(&mut self) -> Result<C, Self::Error>;
    fn bind(address: &str) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

pub trait ClientTrait<C: Connection> {
    type Error: std::fmt::Debug;

    fn connect(address: &str) -> Result<C, Self::Error>;
}

mod tcp;

pub use tcp::TCPClient;
pub use tcp::TCPConnection;
pub use tcp::TCPServer;
