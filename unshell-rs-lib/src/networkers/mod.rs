/// This is the lowset-level data transmission type

pub trait Connection: Send + Sync {
    type Error: std::fmt::Debug;

    fn get_info(&self) -> String;
    fn is_alive(&self) -> bool;

    fn read(&mut self) -> Result<String, Self::Error>;
    fn write(&mut self, data: &str) -> Result<(), Self::Error>;
}

pub trait ServerTrait<C: Connection> {
    type Error: std::fmt::Debug;

    fn get_info(&self) -> String;
    fn accept(&self) -> Result<C, Self::Error>;
    fn bind(address: &SocketAddr) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

pub trait ClientTrait<C: Connection + Sized> {
    type Error: std::fmt::Debug;

    fn connect(address: &SocketAddr) -> Result<C, Self::Error>;
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

    thread::spawn(move || {
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
    });
}

pub fn run_listener<S, C, R>(server: S, on_connect_callback: R)
/*-> Arc<Mutex<Vec<C>>>*/
where
    S: ServerTrait<C> + Sync + Send + 'static,
    C: Connection + 'static,
    R: Fn(C) + Sync + Send + 'static,
{
    info!("Started listener {}", server.get_info());
    // let clients: Arc<Mutex<Vec<C>>> = Arc::new(Mutex::new(Vec::new()));
    // let clients_clone = Arc::clone(&clients);

    thread::spawn(move || {
        loop {
            match server.accept() {
                Ok(conn) => {
                    info!("New connection ({})", conn.get_info());

                    on_connect_callback(conn);

                    // OnConnectCallback::on_connect(&mut on_connect_callback, conn);
                    // let mut clients_lock = clients_clone.lock().unwrap();
                    // clients_lock.push(conn);
                }
                Err(e) => {
                    error!("Failed to accept connection: {:?}", e);
                }
            }
        }
    });
}

mod tcp;

use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;

pub use tcp::TCPClient;
pub use tcp::TCPConnection;
pub use tcp::TCPServer;
