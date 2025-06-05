use std::sync::{Arc, Mutex};

use crate::{
    listeners::client::Client,
    networkers::{Connection, ServerTrait},
};

pub struct Listener<S, C> {
    pub server: Arc<Mutex<S>>,
    pub clients: Arc<Mutex<Vec<Client<C>>>>,
}

impl<S, C> Listener<S, C> {
    pub fn new(server: S) -> Self {
        Self {
            server: Arc::new(Mutex::new(server)),
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn run_listener(&mut self) -> Result<(), Box<dyn std::error::Error>>
    where
        S: ServerTrait<C>,
        C: Connection + 'static,
        S::Error: std::error::Error + 'static,
        C::Error: std::error::Error + 'static,
    {
        loop {
            let mut conn_lock = self.server.lock().unwrap();

            match conn_lock.accept() {
                Ok(conn) => {
                    let mut clients_lock = self.clients.lock().unwrap();
                    clients_lock.push(Client::new(conn));
                }
                Err(e) => {
                    error!("Failed to accept connection: {:?}", e);
                }
            }
        }
    }
}
