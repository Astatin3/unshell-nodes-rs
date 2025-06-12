use std::net::SocketAddr;

use unshell_rs_lib::{
    Error,
    nodes::{ConnectionConfig, Node},
};

use crate::C2Packet;

pub fn run_endpoint(socket: SocketAddr) -> Result<(), Error> {
    let node = Node::<C2Packet>::run_node(
        "Server".to_string(),
        vec![],
        vec![ConnectionConfig {
            socket,
            layers: vec![],
        }],
    )?;

    loop {
        match node.rx.recv()? {
            C2Packet::Aa => {
                info!("1");
            }
            C2Packet::Bb => {
                info!("2");
            }
            C2Packet::Cc => {
                info!("3");
            }
        }
    }
}
