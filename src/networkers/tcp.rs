use std::{
    io::{self, BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use crate::networkers::{ClientTrait, Connection, ServerTrait};

pub struct TCPConnection {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

impl Connection for TCPConnection {
    type Error = io::Error;

    fn read(&mut self) -> Result<String, Self::Error> {
        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        Ok(line.trim_end().to_string())
    }

    fn write(&mut self, data: &str) -> Result<(), Self::Error> {
        writeln!(self.stream, "{}", data)?;
        self.stream.flush()
    }
}

pub struct TCPServer {
    listener: TcpListener,
}

impl ServerTrait<TCPConnection> for TCPServer {
    type Error = io::Error;

    fn accept(&mut self) -> Result<TCPConnection, Self::Error> {
        let (stream, _) = self.listener.accept()?;
        let reader = BufReader::new(stream.try_clone()?);
        Ok(TCPConnection { stream, reader })
    }

    fn bind(address: &str) -> Result<Self, Self::Error> {
        let listener = TcpListener::bind(address)?;
        Ok(Self { listener })
    }
}

pub struct TCPClient;

impl ClientTrait<TCPConnection> for TCPClient {
    type Error = io::Error;

    fn connect(address: &str) -> Result<TCPConnection, Self::Error> {
        let stream = TcpStream::connect(address)?;
        let reader = BufReader::new(stream.try_clone()?);
        Ok(TCPConnection { stream, reader })
    }
}
