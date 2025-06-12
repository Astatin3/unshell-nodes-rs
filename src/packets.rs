use bincode::{Decode, Encode};

#[derive(Debug, Encode, Decode, Clone)]
pub enum C2Packet {
    Aa,
    Bb,
    Cc,
}
