#[macro_use]
extern crate log;

mod client;
mod endpoint;
mod packets;

pub use client::Cli;

pub use endpoint::run_endpoint;
pub use packets::C2Packet;

// pub use client::UnshellClient;
// pub use client::UnshellGui;
// pub use server::UnshellServer;
