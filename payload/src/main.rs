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

// Generic client function
pub fn run_client<C, Cl>(address: &str) -> Result<(), Box<dyn std::error::Error>>
where
    Cl: ClientTrait<C>,
    C: Connection + 'static,
    Cl::Error: std::error::Error + 'static,
    C::Error: std::error::Error + 'static,
{
    let recv_conn = Arc::new(Mutex::new(Cl::connect(address)?));
    let transmit_vec: Arc<Mutex<Vec<Packet>>> = Arc::new(Mutex::new(Vec::new()));

    let transmit_conn = Arc::clone(&recv_conn);
    let transmit_vec_clone = Arc::clone(&transmit_vec);

    thread::spawn(move || {
        loop {
            let mut transmit_vec_lock = transmit_vec.lock().unwrap();
            if transmit_vec_lock.len() > 0 {
                let mut conn_lock = recv_conn.lock().unwrap();
                if let Ok(json) = serde_json::to_string(&transmit_vec_lock.pop().unwrap()) {
                    conn_lock.write(&json).expect("Failed to send packet!");
                }
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        }
    });

    loop {
        let mut conn_lock = transmit_conn.lock().unwrap();
        let data = conn_lock.read();
        drop(conn_lock);
        match data {
            Ok(data_json) => {
                if data_json.is_empty() {
                    continue;
                }
                let packet = serde_json::from_str::<Packet>(data_json.as_str());
                println!("{:?}", packet);
            }
            Err(e) => {
                eprintln!("Error reading, {}", e);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    run_client::<TCPConnection, TCPClient>("127.0.0.1:3000")?;

    Ok(())
}
