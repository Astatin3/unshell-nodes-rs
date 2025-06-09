use crate::{
    layers::Base64Layer,
    networkers::{Connection, ProtocolLayer},
};

type Error = Box<dyn std::error::Error>;

// 4-Way Handshake Layer
pub struct HandshakeLayer<C: Connection> {
    inner: C,
    finished_handshake: bool,
}

impl<C: Connection> Connection for HandshakeLayer<C> {
    fn get_info(&self) -> String {
        format!("handshake->{}", self.inner.get_info())
    }

    fn is_alive(&self) -> bool {
        self.inner.is_alive()
    }

    fn read(&mut self) -> Result<String, Error> {
        if !self.finished_handshake {
            return Err("NotComplete".into());
        }
        self.inner.read()
    }

    fn write(&mut self, data: &str) -> Result<(), Error> {
        if !self.finished_handshake {
            return Err("NotComplete".into());
        }
        self.inner.write(data)
    }
}

impl<C: Connection + 'static> ProtocolLayer<C> for HandshakeLayer<C> {
    fn new(inner: C) -> Result<Self, Error> {
        Ok(HandshakeLayer {
            inner,
            finished_handshake: false,
        })
    }

    fn initialize_client(&mut self) -> Result<(), Error> {
        println!("Starting client handshake...");

        // Step 1: Client sends SYN
        self.inner.write("SYN")?;
        println!("Client: Sent SYN");

        // Step 2: Client receives SYN-ACK
        let response = self.inner.read()?;
        if response != "SYN-ACK" {
            return Err(format!("Expected SYN-ACK, got: {}", response).into());
        }
        println!("Client: Received SYN-ACK");

        // Step 3: Client sends ACK
        self.inner.write("ACK")?;
        println!("Client: Sent ACK");

        // Step 4: Client receives FIN (final confirmation)
        let response = self.inner.read()?;
        if response != "FIN" {
            return Err(format!("Expected FIN, got: {}", response).into());
        }
        println!("Client: Received FIN - Handshake complete!");

        self.finished_handshake = true;
        Ok(())
    }

    fn initialize_server(&mut self) -> Result<(), Error> {
        println!("Starting server handshake...");

        // Step 1: Server receives SYN
        let request = self.inner.read()?;
        if request != "SYN" {
            return Err(format!("Expected SYN, got: {}", request).into());
        }
        println!("Server: Received SYN");

        // Step 2: Server sends SYN-ACK
        self.inner.write("SYN-ACK")?;
        println!("Server: Sent SYN-ACK");

        // Step 3: Server receives ACK
        let response = self.inner.read()?;
        if response != "ACK" {
            return Err(format!("Expected ACK, got: {}", response).into());
        }
        println!("Server: Received ACK");

        // Step 4: Server sends FIN (final confirmation)
        self.inner.write("FIN")?;
        println!("Server: Sent FIN - Handshake complete!");

        self.finished_handshake = true;
        Ok(())
    }
}
