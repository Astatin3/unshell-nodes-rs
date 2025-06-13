use crossbeam_channel::{Receiver, Sender};

use crate::networkers::Connection;

pub struct Stream {
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
}

impl Connection for Stream {
    fn get_info(&self) -> String {
        "unrouted".to_string()
    }

    fn is_alive(&self) -> bool {
        true
    }

    fn read(&mut self) -> Result<Vec<u8>, crate::Error> {
        Ok(self.rx.recv()?)
    }

    fn write(&mut self, data: &[u8]) -> Result<(), crate::Error> {
        Ok(self.tx.send(data.to_vec())?)
    }

    fn try_clone(&self) -> Result<Box<dyn Connection + Send + Sync>, crate::Error> {
        todo!()
    }
}

impl Stream {
    pub fn new(tx: Sender<Vec<u8>>, rx: Receiver<Vec<u8>>) -> Self {
        Self { tx, rx }
    }
}
