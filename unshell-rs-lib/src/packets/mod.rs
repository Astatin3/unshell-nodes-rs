use serde::{Deserialize, Serialize};

mod sysinfo;

#[derive(Serialize, Deserialize, Debug)]
pub enum Packet {
    Heartbeat,
    Sysinfo(sysinfo::Sysinfo),
}
