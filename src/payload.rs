use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    networkers::{ClientTrait, Connection},
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

    // loop {
    //     let mut input = String::new();
    //     stdin.read_line(&mut input)?;
    //     let input = input.trim();

    //     if input == "quit" {
    //         conn.write(input)?;
    //         break;
    //     }

    //     if !input.is_empty() {
    //         conn.write(input)?;

    //         match conn.read() {
    //             Ok(response) => println!("Server: {}", response),
    //             Err(e) => {
    //                 eprintln!("Failed to read response: {:?}", e);
    //                 break;
    //             }
    //         }
    //     }
    // }
}
