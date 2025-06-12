use std::{io::Write, net::SocketAddr};

use unshell_rs_lib::{
    Error,
    nodes::{ConnectionConfig, Node},
};

use crate::C2Packet;

pub struct Cli;

impl Cli {
    pub fn connect(socket: SocketAddr) -> Result<(), Error> {
        // let mut client = build_client(TCPClient::connect(&addr)?, vec![])?;

        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        let node = Node::<C2Packet>::run_node(
            "Client".to_string(),
            vec![ConnectionConfig {
                socket,
                layers: vec![],
            }],
            vec![],
        )?;

        // let mut client_clone = client.try_clone()?;
        // thread::spawn(move || {
        //     // let data = client.read()?;

        //     let packet = Packets::decode(client_clone.read().unwrap().as_str()).unwrap();

        //     match packet {
        //         Packets::UpdateConnections(items) => {
        //             for item in items {
        //                 println!("{}", item);
        //             }
        //         }
        //         Packets::UpdateRoutes(items) => {
        //             for item in items {
        //                 println!("{}", item);
        //             }
        //         }
        //         _ => {
        //             client_clone
        //                 .write(
        //                     Packets::Error(PacketError::UnsupportedType)
        //                         .encode()
        //                         .unwrap()
        //                         .as_str(),
        //                 )
        //                 .unwrap();
        //             warn!("Invalid packet: {:?}", packet)
        //         }
        //     }
        // });

        let selected_node: Option<usize> = None;

        loop {
            print!("> ");
            stdout.flush()?;

            let mut input = String::new();
            stdin.read_line(&mut input)?;
            let input = input.trim();

            let mut node_state = node.state.lock().unwrap();

            let mut split = input.split(" ");

            match split.next().unwrap() {
                "nodes" => {
                    for (i, node) in node_state.get_all_nodes().iter().enumerate() {
                        println!("{} -> {}", i, node);
                    }
                }
                "ping" => {
                    // if split.count().clone() <= 1 {
                    //     warn!("You must specify an option");
                    //     continue;
                    // }

                    if let Ok(i) = str::parse::<usize>(split.next().unwrap()) {
                        let nodes = node_state.get_all_nodes();
                        let node = nodes.get(i).unwrap().clone();
                        node_state.send_unrouted(node, &C2Packet::Aa).unwrap();
                    } else {
                        println!("");
                    }
                }
                _ => {
                    warn!("Invalid command!")
                }
            }

            // client.write(input)?;
        }
    }
}
