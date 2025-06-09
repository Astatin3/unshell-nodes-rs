use std::{
    io::{self, BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
};

use crossbeam_channel::{Receiver, Sender};
use serde::{Serialize, de::DeserializeOwned};

use crate::networkers::{AsyncConnection, ClientTrait, Connection, ServerTrait};

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

impl AsyncConnection<TCPConnection> for TCPConnection {
    type Error = io::Error;

    fn as_async<T: Serialize + DeserializeOwned + Send + 'static>(
        connection: TCPConnection,
    ) -> (Sender<T>, Receiver<T>) {
        let (send_tx, send_rx) = crossbeam_channel::unbounded::<T>();
        let (recv_tx, recv_rx) = crossbeam_channel::unbounded::<T>();

        thread::spawn(move || {
            let mut reader = connection.reader;

            let mut read = || -> Result<String, Self::Error> {
                let mut line = String::new();
                let _ = reader.read_line(&mut line)?;

                Ok(line.trim_end().to_string())
            };

            loop {
                if let Ok(data) = read() {
                    if data.is_empty() {
                        break;
                    }
                    info!("Got {}", data);
                    if let Ok(decoded) = serde_json::from_str::<T>(&data) {
                        if let Err(e) = send_tx.send(decoded) {
                            error!("Got error: {}", e);
                        }
                    }
                }
            }
        });

        thread::spawn(move || {
            let mut stream = connection.stream;

            let mut write = |data: String| -> Result<(), Self::Error> {
                writeln!(stream, "{}", data)?;
                stream.flush()?;
                Ok(())
            };

            loop {
                if let Ok(data) = recv_rx.recv() {
                    if let Ok(encoded) = serde_json::to_string(&data) {
                        info!("Write {}", encoded);
                        if let Err(e) = write(encoded) {
                            error!("Got error: {}", e);
                        }
                    }
                }
            }
        });

        (recv_tx, send_rx)
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
