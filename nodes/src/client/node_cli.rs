use std::{
    io::{Read, Write, stdin, stdout},
    thread,
};

use clap::{Parser, Subcommand};
use unshell_rs_lib::{
    Error,
    networkers::Connection,
    nodes::{NodeContainer, Stream},
};

use crate::client::cli::{Cli, CommandHolder};

pub struct NodeCli {
    node: NodeContainer,
    subcommand: Option<Box<dyn Cli>>,
}

#[derive(Debug, Subcommand)]
pub enum NodeCliCommands {
    /// List out connected nodes
    Nodes,
    /// Send a ping to a remote node
    Ping { n: usize },
    // /// Attempt to create a shell at a remote node
    // Sh { n: usize },
    /// Attempt to create a shell at a remote node
    Stream { n: usize },
}

impl Cli for NodeCli {
    fn name(&self) -> String {
        "Local".to_string()
    }
    fn parse(&mut self, input: Vec<String>) -> Result<(), Error> {
        if let Some(subcommand) = &mut self.subcommand {
            return subcommand.parse(input);
        }
        let parsed_command = CommandHolder::<NodeCliCommands>::try_parse_from(input)?;

        let node_ids = self.node.get_nodes();

        match parsed_command.command {
            NodeCliCommands::Nodes => {
                info!("N | Name");
                for (i, node) in node_ids.iter().enumerate() {
                    info!("[{}] {}", i + 1, node);
                }
            }
            NodeCliCommands::Ping { .. } => {
                // if split.count().clone() <= 1 {
                //     warn!("You must specify an option");
                //     continue;
                // }

                // if n <= 0 {
                //     warn!("Node id must be greater than zero");
                // } else if n > node_ids.len() {
                //     warn!("Node id {} is out of maximum range {}", n, node_ids.len());
                // } else {
                //     let start = Instant::now();
                //     let node = node_ids.get(n - 1).unwrap().clone();
                //     self.node.send_unrouted(&node, &C2Packet::Ping).unwrap();
                //     info!("Sent ping...");

                //     let (_, packet) = self.node.read()?;
                //     match packet {
                //         C2Packet::Pong => {
                //             // if src != nod
                //             info!(
                //                 "Pong! Latency: {}ms",
                //                 (start.elapsed().as_micros() as f32) / 1000.
                //             );
                //         }
                //         _ => {
                //             error!("Got incorrect packet: {:?}", packet);
                //         }
                //     }

                //     // node_state = self.node.state.lock().unwrap();
                // }
            }
            NodeCliCommands::Stream { n } => {
                if n <= 0 {
                    warn!("Node id must be greater than zero");
                } else if n > node_ids.len() {
                    warn!("Node id {} is out of maximum range {}", n, node_ids.len());
                } else {
                    let node_id = node_ids.get(n - 1).unwrap().clone();

                    let stream = self.node.create_stream_block(node_id)?;

                    self.run_pty(stream)?;
                }
            }
        }

        Ok(())
    }
}

impl NodeCli {
    pub fn new(node: NodeContainer) -> Self {
        Self {
            node,
            subcommand: None,
        }
    }

    pub fn run_pty(&mut self, mut stream: Stream) -> Result<(), Error> {
        let mut stream_clone = stream.try_clone()?;

        // Thread to read from stdin and write to TCP stream
        let stdin_to_tcp = thread::spawn(move || {
            let mut stdin = stdin();
            let mut buffer = [0u8; 1024];
            loop {
                match stdin.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        if stream.write(&buffer[..n]).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error reading from stdin: {}", e);
                        break;
                    }
                }
            }
        });

        // Thread to read from TCP stream and write to stdout
        let tcp_to_stdout = thread::spawn(move || {
            loop {
                let data = stream_clone.read().unwrap();
                if stdout().write_all(&data).is_err() {
                    break;
                }
                stdout().flush().ok();
            }
        });

        // Wait for either thread to finish
        let _ = stdin_to_tcp.join();
        let _ = tcp_to_stdout.join();

        error!("Disconnected from server");

        Ok(())
        // pty_pair.Ok(())
    }
}
