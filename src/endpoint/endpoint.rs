use std::net::SocketAddr;

use unshell_rs_lib::{
    C2Packet, Error,
    nodes::{ConnectionConfig, NodeContainer},
};

pub fn run_endpoint(socket: SocketAddr) -> Result<(), Error> {
    let node = NodeContainer::connect(
        "Server".to_string(),
        vec![],
        vec![ConnectionConfig {
            socket,
            layers: vec![],
        }],
    )?;

    loop {
        let (src, packet) = node.read_packet()?;
        match packet {
            C2Packet::Ping => {
                info!("Ping from {}!", src);
                node.send_unrouted(&src, &C2Packet::Pong)?;
                // (&mut node.state.lock().unwrap()).send_unrouted(src, &C2Packet::Pong)?;
            }
            C2Packet::Pong => {
                info!("Pong!");
            }
            _ => {}
        }
    }
}
