use std::{net::SocketAddr, thread};

use crate::{
    Error,
    layers::LayerConfig,
    networkers::{Connection, ServerTrait, TCPConnection, TCPServer, run_listener},
};

pub struct Node;

impl Node {
    pub fn run(addr: SocketAddr) -> Result<(), Error> {
        let layers = vec![LayerConfig::Handshake, LayerConfig::Base64];

        run_listener(
            TCPServer::bind(&addr)?,
            layers,
            |connection: Box<dyn Connection + Send + 'static>| {
                thread::spawn(move || {
                    let mut connection = connection;

                    loop {
                        if let Ok(data) = connection.read() {
                            if !connection.is_alive() {
                                warn!("{} Disconnected!", connection.get_info());
                                break;
                            }
                            println!("Data: {}", data);
                        }
                    }
                });
            },
        );

        Ok(())
    }
}
