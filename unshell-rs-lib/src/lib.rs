#[macro_use]
extern crate log;

pub type Error = Box<dyn std::error::Error>;

static BINCODE_CONFIG: bincode::config::Configuration = bincode::config::standard();

pub mod layers;
pub mod networkers;
pub mod nodes;
