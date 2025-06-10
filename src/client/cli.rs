use std::{io::Write, net::SocketAddr, thread};

use unshell_rs_lib::{
    Error,
    connection::{PacketError, Packets},
    layers::build_client,
    networkers::{ClientTrait, Connection, TCPClient},
};
pub struct Cli;

impl Cli {
    pub fn connect(addr: SocketAddr) -> Result<(), Error> {
        let mut client = build_client(TCPClient::connect(&addr)?, vec![])?;

        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        let mut client_clone = client.try_clone()?;
        thread::spawn(move || {
            // let data = client.read()?;

            let packet = Packets::decode(client_clone.read().unwrap().as_str()).unwrap();

            match packet {
                Packets::UpdateConnections(items) => {
                    for item in items {
                        println!("{}", item);
                    }
                }
                Packets::UpdateRoutes(items) => {
                    for item in items {
                        println!("{}", item);
                    }
                }
                _ => {
                    client_clone
                        .write(
                            Packets::Error(PacketError::UnsupportedType)
                                .encode()
                                .unwrap()
                                .as_str(),
                        )
                        .unwrap();
                    warn!("Invalid packet: {:?}", packet)
                }
            }
        });

        loop {
            print!("> ");
            stdout.flush()?;

            let mut input = String::new();
            stdin.read_line(&mut input)?;
            let input = input.trim();

            match input.split(" ").nth(0).unwrap() {
                "clients" => {
                    client.write(Packets::GetConnections.encode()?.as_str())?;
                }
                "routes" => {
                    client.write(Packets::GetRoutes.encode()?.as_str())?;
                }
                _ => {
                    warn!("Invalid command!")
                }
            }

            // client.write(input)?;
        }
    }
}
