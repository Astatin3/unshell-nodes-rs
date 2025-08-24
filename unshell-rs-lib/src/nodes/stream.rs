use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crossbeam_channel::{Receiver, Sender};

use crate::{Error, networkers::Connection, nodes::Node, packets::TransportLayerPacket};

pub struct Stream {
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
    closed: Arc<AtomicBool>,
}

impl Connection for Stream {
    fn get_info(&self) -> String {
        "unrouted".to_string()
    }

    fn is_alive(&self) -> bool {
        self.closed.load(Ordering::Relaxed)
    }

    fn read(&mut self) -> Result<Vec<u8>, crate::Error> {
        if self.closed.load(Ordering::Relaxed) {
            Err("Connection closed".into())
        } else {
            Ok(self.rx.recv()?)
        }
    }

    fn write(&mut self, data: &[u8]) -> Result<(), crate::Error> {
        if self.closed.load(Ordering::Relaxed) {
            Err("Connection closed".into())
        } else {
            Ok(self.tx.send(data.to_vec())?)
        }
    }

    fn try_clone(&self) -> Result<Box<dyn Connection + Send + Sync>, crate::Error> {
        Ok(Box::new(Self {
            tx: self.tx.clone(),
            rx: self.rx.clone(),
            closed: Arc::clone(&self.closed),
        }))
    }
}

impl Stream {
    pub fn ask_create(
        dest: String,
        local_stream_id: usize,
        node: &Node<TransportLayerPacket>,
    ) -> Result<(), Error> {
        info!("Sent to {}", dest);
        node.send_unrouted(
            dest.clone(),
            &TransportLayerPacket::RequestStreamUnrouted {
                stream_id: local_stream_id,
            },
        )?;

        Ok(())

        // Self::create_handle(dest, local_stream_id, remote_stream_id, node)
    }

    pub fn respond_create(
        dest: String,
        local_stream_id: usize,
        remote_stream_id: usize,
        node: &Node<TransportLayerPacket>,
    ) -> Result<(), Error> {
        node.send_unrouted(
            dest.clone(),
            &&TransportLayerPacket::AckStreamUnrouted {
                ack_stream_id: remote_stream_id,
                stream_id: local_stream_id,
            },
        )?;

        Ok(())
    }

    pub fn create_handle(
        dest: String,
        local_stream_id: usize,
        remote_stream_id: usize,
    ) -> Result<(Self, StreamHandle), Error> {
        let (recv_tx, recv_rx) = crossbeam_channel::unbounded();
        let (send_tx, send_rx) = crossbeam_channel::unbounded();

        let closed = Arc::new(AtomicBool::new(false));

        let handle = StreamHandle {
            dest,
            tx: recv_tx,
            rx: send_rx,
            local_stream_id,
            remote_stream_id,
            closed: Arc::clone(&closed),
        };

        Ok((
            Self {
                tx: send_tx,
                rx: recv_rx,
                closed,
            },
            handle,
        ))
    }

    // pub fn new(tx: Sender<Vec<u8>>, rx: Receiver<Vec<u8>>) -> Self {
    //     Self { tx, rx }
    // }
}

pub struct StreamHandle {
    pub dest: String,
    pub local_stream_id: usize,
    pub remote_stream_id: usize,

    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
    closed: Arc<AtomicBool>,
}

impl StreamHandle {
    pub fn send(&self, data: Vec<u8>) -> Result<(), Error> {
        Ok(self.tx.send(data)?)
    }
    pub fn has_content(&self) -> bool {
        self.rx.len() > 0
    }
    pub fn recv(&self) -> Result<Vec<u8>, Error> {
        Ok(self.rx.recv()?)
    }
    pub fn clone(&self) -> Result<Self, Error> {
        Ok(Self {
            dest: self.dest.clone(),
            local_stream_id: self.local_stream_id.clone(),
            remote_stream_id: self.remote_stream_id.clone(),
            tx: self.tx.clone(),
            rx: self.rx.clone(),
            closed: Arc::clone(&self.closed),
        })
    }
    pub fn close(self) {
        drop(self.tx);
        drop(self.rx);
        self.closed.store(false, Ordering::Relaxed);
    }
}
