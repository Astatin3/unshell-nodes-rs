use std::{error::Error, io::Write, net::SocketAddr};

use unshell_rs_lib::{
    layers::{LayerConfig, build_client},
    networkers::{ClientTrait, Connection, TCPClient},
};

use crate::client;

pub struct Cli;

impl Cli {
    pub fn connect(addr: SocketAddr) -> Result<(), Box<dyn Error>> {
        let mut client = build_client(
            TCPClient::connect(&addr)?,
            vec![LayerConfig::Handshake, LayerConfig::Base64],
        )?;

        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        loop {
            print!("> ");
            stdout.flush()?;

            let mut input = String::new();
            stdin.read_line(&mut input)?;
            let input = input.trim();

            client.write(input)?;
        }
    }
}
