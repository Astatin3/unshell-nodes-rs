use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::networkers::{Connection, ProtocolLayer};

type Error = Box<dyn std::error::Error>;

// 4-Way Handshake Layer
pub struct HandshakeLayer {
    inner: Box<dyn Connection>,
    finished_handshake: Arc<AtomicBool>,
}

impl Connection for HandshakeLayer {
    fn get_info(&self) -> String {
        format!("handshake->{}", self.inner.get_info())
    }

    fn is_alive(&self) -> bool {
        self.inner.is_alive()
    }

    fn read(&mut self) -> Result<String, Error> {
        if !self.finished_handshake.load(Ordering::Relaxed) {
            return Err("NotComplete".into());
        }
        self.inner.read()
    }

    fn write(&mut self, data: &str) -> Result<(), Error> {
        if !self.finished_handshake.load(Ordering::Relaxed) {
            return Err("NotComplete".into());
        }
        self.inner.write(data)
    }

    fn try_clone(&self) -> Result<Box<dyn Connection + Send + Sync>, crate::Error> {
        Ok(Box::new(Self {
            inner: self.inner.try_clone()?,
            finished_handshake: Arc::clone(&self.finished_handshake.clone()),
        }))
    }
}

impl ProtocolLayer for HandshakeLayer {
    fn new(inner: Box<dyn Connection>) -> Result<Self, Error> {
        Ok(HandshakeLayer {
            inner,
            finished_handshake: Arc::new(AtomicBool::new(false)),
        })
    }

    fn initialize_client(&mut self) -> Result<(), Error> {
        // Step 1: Client sends SYN
        self.inner.write("SYN")?;

        // Step 2: Client receives SYN-ACK
        let response = self.inner.read()?;
        if response != "SYN-ACK" {
            return Err(format!("Expected SYN-ACK, got: {}", response).into());
        }

        // Step 3: Client sends ACK
        self.inner.write("ACK")?;

        // Step 4: Client receives FIN (final confirmation)
        let response = self.inner.read()?;
        if response != "FIN" {
            return Err(format!("Expected FIN, got: {}", response).into());
        }

        info!("Handshake complete!");

        self.finished_handshake.swap(true, Ordering::Relaxed);
        Ok(())
    }

    fn initialize_server(&mut self) -> Result<(), Error> {
        // Step 1: Server receives SYN
        let request = self.inner.read()?;
        if request != "SYN" {
            return Err(format!("Expected SYN, got: {}", request).into());
        }
        // Step 2: Server sends SYN-ACK
        self.inner.write("SYN-ACK")?;

        // Step 3: Server receives ACK
        let response = self.inner.read()?;
        if response != "ACK" {
            return Err(format!("Expected ACK, got: {}", response).into());
        }

        // Step 4: Server sends FIN (final confirmation)
        self.inner.write("FIN")?;
        info!("Handshake complete!");

        self.finished_handshake.swap(true, Ordering::Relaxed);
        Ok(())
    }
}
