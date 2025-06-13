use std::time::Instant;

use clap::{Parser, Subcommand};
use portable_pty::{PtySize, native_pty_system};
use unshell_rs_lib::{C2Packet, Error, nodes::NodeContainer};

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
    /// Attempt to create a shell at a remote node
    Sh { n: usize },
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
            NodeCliCommands::Ping { n } => {
                // if split.count().clone() <= 1 {
                //     warn!("You must specify an option");
                //     continue;
                // }

                if n <= 0 {
                    warn!("Node id must be greater than zero");
                } else if n > node_ids.len() {
                    warn!("Node id {} is out of maximum range {}", n, node_ids.len());
                } else {
                    let start = Instant::now();
                    let node = node_ids.get(n - 1).unwrap().clone();
                    self.node.send_unrouted(&node, &C2Packet::Ping).unwrap();
                    info!("Sent ping...");

                    let (_, packet) = self.node.read_packet()?;
                    match packet {
                        C2Packet::Pong => {
                            // if src != nod
                            info!(
                                "Pong! Latency: {}ms",
                                (start.elapsed().as_micros() as f32) / 1000.
                            );
                        }
                        _ => {
                            error!("Got incorrect packet: {:?}", packet);
                        }
                    }

                    // node_state = self.node.state.lock().unwrap();
                }
            }
            NodeCliCommands::Sh { n } => {
                if n <= 0 {
                    warn!("Node id must be greater than zero");
                } else if n > node_ids.len() {
                    warn!("Node id {} is out of maximum range {}", n, node_ids.len());
                } else {
                    let node_id = node_ids.get(n - 1).unwrap().clone();
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

    pub fn run_pty(&mut self) -> Result<(), Error> {
        let pty_system = native_pty_system();
        let pty_pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        Ok(())

        // pty_pair.Ok(())
    }
}
