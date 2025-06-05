use std::error::Error;

use unshell_rs::{
    listeners::Listener,
    networkers::{ServerTrait, TCPServer},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut server = Listener::new(TCPServer::bind("0.0.0.0:3000")?);

    server.run_listener()?;

    Ok(())
}
