use std::fmt::Debug;

use bincode::{Decode, Encode, config::Configuration};

use crate::Error;

#[derive(Debug, Encode, Decode, Clone)]
pub enum Packets {
    SyncUUID(String),
    Update {
        routes: Vec<String>,
    },
    Disconnect {
        routes: Vec<String>,
    },

    // Send single data packet without routing details
    DataUnrouted {
        src: String,
        dest: String,
        data: Vec<u8>,
    },
    // Send single data packet with routing details
    DataRouted {
        path: Vec<String>,
        data: Vec<u8>,
    },

    // DataStreamRouted {
    //     path: Vec<String>,
    //     data: Vec<u8>,
    // },
    ErrorNameExists,
}

impl Packets {
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        encode_vec(self)
    }
    pub fn decode(data: &[u8]) -> Result<Self, Error> {
        decode_vec(data)
    }
}

pub fn encode_vec<P>(object: &P) -> Result<Vec<u8>, Error>
where
    P: Encode + Decode<()> + Debug + Clone + 'static,
{
    Ok(bincode::encode_to_vec(object, crate::BINCODE_CONFIG)?)
}

pub fn decode_vec<P>(data: &[u8]) -> Result<P, Error>
where
    P: Encode + Decode<()> + Debug + Clone + 'static,
{
    let (decoded, _) =
        bincode::decode_from_slice::<P, Configuration>(&data[..], crate::BINCODE_CONFIG)?;

    Ok(decoded)
}
