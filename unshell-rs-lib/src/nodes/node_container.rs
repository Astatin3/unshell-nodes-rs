use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use crossbeam_channel::{Receiver, Sender};

use crate::{
    C2Packet, Error,
    nodes::{
        ConnectionConfig, Node, Stream,
        node::NodeState,
        packets::{decode_vec, encode_vec},
    },
    packets::TransportLayerPacket,
};

type Streams = Arc<Mutex<HashMap<(usize, String), Option<(Stream, Sender<Vec<u8>>)>>>>;

pub struct NodeContainer {
    streams: Streams,
    state: Arc<Mutex<NodeState<TransportLayerPacket>>>,
    spontanious_rx: Receiver<(String, C2Packet)>,
}

impl NodeContainer {
    pub fn connect(
        id: String,
        clients: Vec<ConnectionConfig>,
        listeners: Vec<ConnectionConfig>,
    ) -> Result<Self, Error> {
        let node = Node::run_node(id, clients, listeners)?;
        let streams = Arc::new(Mutex::new(HashMap::new()));
        let (spontanious_tx, spontanious_rx) = crossbeam_channel::unbounded();

        let s = Self {
            streams: Arc::clone(&streams),
            state: Arc::clone(&node.state),
            spontanious_rx,
        };

        // Start node listening thread
        thread::spawn(move || {
            loop {
                if let Err(e) = Self::node_listening_thread(&node, &streams, &spontanious_tx) {
                    error!("Got error: {}", e);
                }
            }
        });

        Ok(s)
    }

    fn node_listening_thread(
        node: &Node<TransportLayerPacket>,
        streams: &Streams,
        spontanious_tx: &Sender<(String, C2Packet)>,
    ) -> Result<(), Error> {
        let (src, packet) = node.rx.recv()?;

        match packet {
            TransportLayerPacket::RequestStreamUnrouted { stream_id } => {
                let local_stream_id = streams.lock().unwrap().keys().len();
                streams
                    .lock()
                    .unwrap()
                    .insert((local_stream_id, src.clone()), None);
                (&mut node.state.lock().unwrap()).send_unrouted(
                    src,
                    &TransportLayerPacket::AckStreamUnrouted {
                        local_stream_id,
                        remote_stream_id: stream_id,
                    },
                )?;

                Ok(())
            }
            TransportLayerPacket::AckStreamUnrouted {
                local_stream_id,
                remote_stream_id,
            } => {
                let key = &(remote_stream_id, src);
                if let Some(stream_mut) = streams.lock().unwrap().get_mut(&key) {
                    if stream_mut.is_none() {
                        let stream = Self::create_stream(local_stream_id, node, src, stream_mut)?;
                        Ok(())
                    } else {
                        Err(format!("Stream {:?} already exists!", key).into())
                    }
                } else {
                    Err(format!("Could not find stream id by {:?}", key).into())
                }
            }
            TransportLayerPacket::StreamDataUnrouted { stream_id, data } => todo!(),
            TransportLayerPacket::SpontaniousDataUnrouted { data } => {
                spontanious_tx.send((src, decode_vec::<C2Packet>(&data)?))?;
                Ok(())
            }
        }
    }

    fn create_stream(
        remote_stream_id: usize,
        dest: String,
        node: &Node<TransportLayerPacket>,
        stream_mut: &mut Option<(Stream, Sender<Vec<u8>>)>,
    ) -> Result<(), Error> {
        let (recv_tx, recv_rx) = crossbeam_channel::unbounded();
        let (send_tx, send_rx) = crossbeam_channel::unbounded();

        let stream = Stream::new(send_tx, recv_rx);

        let _ = stream_mut.insert((stream, recv_tx));

        thread::spawn(move || {
            loop {
                let packet = send_rx.recv().unwrap();
                (&mut node.state.lock().unwrap())
                    .send_unrouted(
                        dest,
                        &TransportLayerPacket::StreamDataUnrouted {
                            stream_id: remote_stream_id,
                            data: packet,
                        },
                    )
                    .unwrap();
            }
        });

        Ok(())
    }

    pub fn get_nodes(&self) -> Vec<String> {
        self.state.lock().unwrap().get_all_nodes()
    }

    pub fn send_unrouted(&self, dest: &String, data: &C2Packet) -> Result<(), Error> {
        (&mut self.state.lock().unwrap()).send_unrouted(
            dest.clone(),
            &TransportLayerPacket::SpontaniousDataUnrouted {
                data: encode_vec(data)?,
            },
        )?;
        Ok(())
    }

    pub fn read_packet(&self) -> Result<(String, C2Packet), Error> {
        Ok(self.spontanious_rx.recv()?)
    }
}
