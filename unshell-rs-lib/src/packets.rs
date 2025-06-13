use bincode::{Decode, Encode};
use std::fmt::Debug;

#[derive(Debug, Encode, Decode, Clone)]
pub enum TransportLayerPacket {
    RequestStreamUnrouted {
        stream_id: usize,
    },
    AckStreamUnrouted {
        local_stream_id: usize,
        remote_stream_id: usize,
    },
    StreamDataUnrouted {
        stream_id: usize,
        data: Vec<u8>,
    },

    SpontaniousDataUnrouted {
        data: Vec<u8>,
    },
}

#[derive(Debug, Encode, Decode, Clone)]
pub enum C2Packet {
    Ping,
    Pong,

    CreatePTY { width: usize, height: usize },
    PTYData,
}
