use std::{
    io::{self, BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};

use crate::networkers::{ClientTrait, Connection, ServerTrait};

pub struct TCPConnection {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    is_alive: bool,
}

impl Connection for TCPConnection {
    type Error = io::Error;

    fn get_info(&self) -> String {
        format!(
            "tcp://{}",
            if let Ok(addr) = &self.stream.peer_addr() {
                addr.to_string()
            } else {
                "ERROR".to_string()
            }
        )
    }

    fn is_alive(&self) -> bool {
        self.is_alive
    }

    fn read(&mut self) -> Result<String, Self::Error> {
        let mut line = String::new();
        let n = self.reader.read_line(&mut line)?;

        // Stream sends a null buffer if it is disconnected
        if n == 0 {
            self.is_alive = false;
        }

        Ok(line.trim_end().to_string())
    }

    fn write(&mut self, data: &str) -> Result<(), Self::Error> {
        writeln!(self.stream, "{}", data)?;
        self.stream.flush()?;
        Ok(())
    }
}

pub struct TCPServer {
    listener: TcpListener,
}

impl ServerTrait<TCPConnection> for TCPServer {
    type Error = io::Error;

    fn get_info(&self) -> String {
        format!(
            "tcp://{}",
            if let Ok(addr) = &self.listener.local_addr() {
                addr.to_string()
            } else {
                "ERROR".to_string()
            }
        )
    }

    fn accept(&self) -> Result<TCPConnection, Self::Error> {
        let (stream, _) = self.listener.accept()?;
        let reader = BufReader::new(stream.try_clone()?);
        Ok(TCPConnection {
            stream,
            reader,
            is_alive: true,
        })
    }

    fn bind(address: &SocketAddr) -> Result<Self, Self::Error> {
        let listener = TcpListener::bind(address)?;
        Ok(Self { listener })
    }
}

pub struct TCPClient;

impl ClientTrait<TCPConnection> for TCPClient {
    type Error = io::Error;

    fn connect(address: &SocketAddr) -> Result<TCPConnection, Self::Error> {
        let stream = TcpStream::connect(address)?;
        let reader = BufReader::new(stream.try_clone()?);
        let conn = TCPConnection {
            stream,
            reader,
            is_alive: true,
        };
        info!("Connected to {}", conn.get_info());
        Ok(conn)
    }
}
