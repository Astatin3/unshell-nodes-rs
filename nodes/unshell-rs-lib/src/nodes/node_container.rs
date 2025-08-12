use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use crossbeam_channel::{Receiver, Sender};

use crate::{
    Error,
    nodes::{ConnectionConfig, Node, Stream, stream::StreamHandle},
    packets::TransportLayerPacket,
};

type Streams = Arc<Mutex<HashMap<(usize, String), StreamHandle>>>;

pub struct NodeContainer {
    streams: Streams,
    node: Node<TransportLayerPacket>,
    on_stream_rx: Receiver<Stream>,
    // state: Arc<Mutex<NodeState<TransportLayerPacket>>>,
    // spontanious_rx: Receiver<(String, C2Packet)>,
}

impl NodeContainer {
    pub fn connect(
        id: String,
        clients: Vec<ConnectionConfig>,
        listeners: Vec<ConnectionConfig>,
    ) -> Result<Self, Error> {
        let node = Node::<TransportLayerPacket>::run_node(id, clients, listeners)?;
        let streams = Arc::new(Mutex::new(HashMap::new()));
        // let (spontanious_tx, spontanious_rx) = crossbeam_channel::unbounded();
        let (on_stream_tx, on_stream_rx) = crossbeam_channel::unbounded();

        let s = Self {
            streams: Arc::clone(&streams),
            node: node.try_clone()?,
            on_stream_rx,
        };

        let close_rx = node.get_disconnect_rx();
        let stream_clone = Arc::clone(&streams);
        thread::spawn(move || {
            loop {
                let close_uuid = close_rx.recv().unwrap();
                let stream = stream_clone.lock().unwrap();
                let keys = stream.keys();
                for key in keys {
                    if key.1 == close_uuid {
                        warn!("Stream ({}, {}) disconnected!", key.0, key.1);
                        let handle = (&mut stream_clone.lock().unwrap()).remove(key).unwrap();
                        handle.close();
                    }
                }
            }
        });

        // Start node listening thread
        thread::spawn(move || {
            loop {
                if let Err(e) = Self::node_listening_thread(&node, &streams, &on_stream_tx) {
                    error!("Got error: {}", e);
                }
            }
        });

        Ok(s)
    }

    fn node_listening_thread(
        node: &Node<TransportLayerPacket>,
        streams: &Streams,
        on_stream_tx: &Sender<Stream>, // spontanious_tx: &Sender<(String, C2Packet)>,
    ) -> Result<(), Error> {
        // info!("Loop");
        let (src, packet) = node.recv()?;
        info!("Packet: {:?}", packet);

        match packet {
            TransportLayerPacket::RequestStreamUnrouted {
                stream_id: remote_stream_id,
            } => {
                // Create stream ID
                let local_stream_id = streams.lock().unwrap().keys().len();
                // Send response to server including local id and remoe ID
                Stream::respond_create(src.clone(), local_stream_id, remote_stream_id, node)?;

                Self::create_handle_thread(
                    on_stream_tx,
                    streams,
                    node,
                    src,
                    local_stream_id,
                    remote_stream_id,
                )?;
                Ok(())
            }
            TransportLayerPacket::AckStreamUnrouted {
                ack_stream_id,
                stream_id,
            } => {
                Self::create_handle_thread(
                    on_stream_tx,
                    streams,
                    node,
                    src,
                    ack_stream_id,
                    stream_id,
                )?;
                Ok(())
            }
            TransportLayerPacket::StreamDataUnrouted { stream_id, data } => {
                match streams.lock().unwrap().get(&(stream_id, src.clone())) {
                    Some(handle) => Ok(handle.send(data).unwrap()),
                    // Some(_) => Err(format!(
                    //     "Stream {}, {} has not been initilized!",
                    //     stream_id, src
                    // )
                    // .into()),
                    None => Err(format!("Stream {}, {} does not exist!", stream_id, src).into()),
                }
            } // _ => Err(format!("Unsupported packet: {:?}", packet).into()),
        }
    }

    fn create_handle_thread(
        on_stream_tx: &Sender<Stream>,
        streams: &Streams,
        node: &Node<TransportLayerPacket>,
        src: String,
        local_stream_id: usize,
        remote_stream_id: usize,
    ) -> Result<(), Error> {
        info!("Local: {}, Remote: {}", local_stream_id, remote_stream_id);

        // Create stream from local and remote stream handles
        let (stream, handle) =
            Stream::create_handle(src.clone(), local_stream_id, remote_stream_id)?;

        on_stream_tx.send(stream)?;

        // Add the local stream to map
        streams
            .lock()
            .unwrap()
            .insert((local_stream_id, src.clone()), handle.clone()?);

        let node_clone = node.try_clone()?;
        thread::spawn(move || {
            loop {
                let data = handle.recv().unwrap();
                if let Err(e) = node_clone.state().send_unrouted(
                    src.clone(),
                    &TransportLayerPacket::StreamDataUnrouted {
                        stream_id: remote_stream_id,
                        data,
                    },
                ) {
                    error!("Got error: {}", e);
                    break;
                }
            }
        });

        Ok(())
    }

    pub fn create_stream_block(&self, dest: String) -> Result<Stream, Error> {
        let local_stream_id = self.streams.lock().unwrap().keys().len();
        Stream::ask_create(dest.clone(), local_stream_id, &self.node)?;
        Ok(self.on_stream_rx.recv()?)
    }

    pub fn recv_stream(&self) -> Result<Stream, Error> {
        Ok(self.on_stream_rx.recv()?)
    }

    pub fn get_nodes(&self) -> Vec<String> {
        self.node.state().get_all_nodes()
        // self.state.lock().unwrap().get_all_nodes()
    }
}
