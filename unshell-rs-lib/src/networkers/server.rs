use std::sync::Arc;
use std::thread;

use crate::{
    layers::{LayerConfig, create_server_builder},
    networkers::{Connection, ServerTrait},
};

// Helper macros for building layered connections
macro_rules! build_layered_connection {
    ($base:expr) => {
        $base
    };
    ($base:expr, $layer:ty) => {
        <$layer>::new($base)?
    };
    ($base:expr, $layer:ty, $($layers:ty),+) => {
        build_layered_connection!(<$layer>::new($base)?, $($layers),+)
    };
}

pub fn run_listener_state<S, C, R, A>(server: S, on_connect_callback: R, state: Arc<A>)
/*-> Arc<Mutex<Vec<C>>>*/
where
    S: ServerTrait<C> + Sync + Send + 'static,
    C: Connection + 'static,
    R: Fn(C, Arc<A>) + Sync + Send + 'static,
    A: Sync + Send + 'static,
{
    info!("Started listener {}", server.get_info());
    // let clients: Arc<Mutex<Vec<C>>> = Arc::new(Mutex::new(Vec::new()));
    // let clients_clone = Arc::clone(&clients);

    loop {
        match server.accept() {
            Ok(conn) => {
                info!("New connection ({})", conn.get_info());

                on_connect_callback(conn, Arc::clone(&state));

                // OnConnectCallback::on_connect(&mut on_connect_callback, conn);
                // let mut clients_lock = clients_clone.lock().unwrap();
                // clients_lock.push(conn);
            }
            Err(e) => {
                error!("Failed to accept connection: {:?}", e);
            }
        }
    }
}

pub fn run_listener<S, C, R>(server: S, layers: Vec<LayerConfig>, on_connect_callback: R)
/*-> Arc<Mutex<Vec<C>>>*/
where
    S: ServerTrait<C> + Sync + Send + 'static,
    C: Connection + 'static,
    R: Fn(Box<dyn Connection + Send + 'static>) + Sync + Send + 'static,
{
    let layer_builder = create_server_builder::<C>(layers).unwrap();

    info!("Started listener {}", server.get_info());
    // let clients: Arc<Mutex<Vec<C>>> = Arc::new(Mutex::new(Vec::new()));
    // let clients_clone = Arc::clone(&clients);

    loop {
        match server.accept() {
            Ok(conn) => match layer_builder(conn) {
                Ok(conn) => {
                    info!("New connection ({})", conn.get_info());
                    on_connect_callback(conn);
                }
                Err(e) => {
                    error!("Failed to create layers: {:?}", e);
                }
            },
            Err(e) => {
                error!("Failed to accept connection: {:?}", e);
            }
        }
    }
}
