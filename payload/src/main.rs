// #[allow(unsafe_op_in_unsafe_fn)]
// mod execute;

use std::error::Error;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use unshell_rs_lib::{
    networkers::{ClientTrait, Connection, TCPClient, TCPConnection},
    packets::Packet,
};

fn main() -> Result<(), Box<dyn Error>> {
    run_client::<TCPConnection, TCPClient>("127.0.0.1:3000")?;

    Ok(())
}
