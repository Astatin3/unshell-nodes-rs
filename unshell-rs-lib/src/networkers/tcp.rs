use std::{
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    Error,
    networkers::{ClientTrait, Connection, ServerTrait},
};

pub struct TCPConnection {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    is_alive: Arc<AtomicBool>,
}

impl Connection for TCPConnection {
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
        self.is_alive.load(Ordering::Relaxed)
    }

    fn read(&mut self) -> Result<String, Error> {
        let mut line = String::new();
        let n = self.reader.read_line(&mut line)?;

        // Stream sends a null buffer if it is disconnected
        if n == 0 {
            self.is_alive.swap(false, Ordering::Relaxed);
        }

        // println!("Recieved: {}", line.trim_end().to_string());

        Ok(line.trim_end().to_string())
    }

    fn write(&mut self, data: &str) -> Result<(), Error> {
        // println!("Recsent: {}", data);
        writeln!(self.stream, "{}", data)?;
        self.stream.flush()?;
        Ok(())
    }

    fn try_clone(&self) -> Result<Box<dyn Connection + Send + Sync>, Error> {
        Ok(Box::new(Self {
            stream: self.stream.try_clone()?,
            reader: BufReader::new(self.stream.try_clone()?),
            is_alive: Arc::clone(&self.is_alive),
        }))
    }
}

pub struct TCPServer {
    listener: TcpListener,
}

impl ServerTrait<TCPConnection> for TCPServer {
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

    fn accept(&self) -> Result<TCPConnection, Error> {
        let (stream, _) = self.listener.accept()?;
        let reader = BufReader::new(stream.try_clone()?);
        Ok(TCPConnection {
            stream,
            reader,
            is_alive: Arc::new(AtomicBool::new(true)),
        })
    }

    fn bind(address: &SocketAddr) -> Result<Self, Error> {
        let listener = TcpListener::bind(address)?;
        Ok(Self { listener })
    }
}

pub struct TCPClient;

impl ClientTrait<TCPConnection> for TCPClient {
    fn connect(address: &SocketAddr) -> Result<TCPConnection, Error> {
        let stream = TcpStream::connect(address)?;
        let reader = BufReader::new(stream.try_clone()?);
        let conn = TCPConnection {
            stream,
            reader,
            is_alive: Arc::new(AtomicBool::new(true)),
        };
        Ok(conn)
    }
}
