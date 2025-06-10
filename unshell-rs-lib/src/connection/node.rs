use std::{
    f32::consts::PI,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use uuid::Uuid;

use crate::{
    Error,
    connection::{listener::ConnectionConfig, packets::Packets},
    layers::build_client,
    networkers::{ClientTrait, Connection, ServerTrait, TCPClient, TCPServer, run_listener_state},
};

pub struct Node {
    // parent: Box<dyn Connection + Send + Sync>,
    clients: Vec<Client>,
}

pub struct Client {
    connection: Box<dyn Connection + Send>,
    uuid: String,
    route: Vec<String>,
}

impl Client {
    pub fn get_info(&self) -> String {
        format!("{} ({})", self.uuid, self.route.join("->"))
    }
}

fn read(c: &mut Box<dyn Connection + Send>) -> Result<Packets, Error> {
    Packets::decode(c.read()?.as_str())
}

fn write(c: &mut Box<dyn Connection + Send>, packet: Packets) -> Result<(), Error> {
    c.write(packet.encode()?.as_str())
}

impl Node {
    fn run_listeners(
        state: &Arc<Mutex<Self>>,
        listeners: Vec<ConnectionConfig>,
    ) -> Result<(), Error> {
        // Start server listeners
        for listener in listeners {
            run_listener_state(
                TCPServer::bind(&listener.socket)?,
                listener.layers,
                Self::on_listener_client,
                Arc::clone(state),
            );
        }

        Ok(())
    }

    pub fn run_node(
        parent: ConnectionConfig,
        listeners: Vec<ConnectionConfig>,
    ) -> Result<(), Error> {
        let mut parent = build_client(TCPClient::connect(&parent.socket)?, parent.layers)?;

        let state = Arc::new(Mutex::new(Self {
            // parent: parent_clone,
            clients: Vec::new(),
        }));

        Self::run_listeners(&state, listeners)?;

        while parent.is_alive() {
            match read(&mut parent) {
                Ok(packet) => match packet {
                    Packets::GetRoutes => write(
                        &mut parent,
                        Packets::UpdateRoutes(state.lock().unwrap().get_routes()),
                    )?,
                    _ => {}
                },
                Err(e) => {
                    error!("Error: {}", e)
                }
            }
        }

        Ok(())
    }

    pub fn run_master(
        server: ConnectionConfig,
        listeners: Vec<ConnectionConfig>,
    ) -> Result<(), Error> {
        // let mut parent = build_client(TCPClient::connect(&parent.socket)?, parent.layers)?;

        let state = Arc::new(Mutex::new(Self {
            // parent: parent_clone,
            clients: Vec::new(),
        }));

        run_listener_state(
            TCPServer::bind(&server.socket)?,
            server.layers,
            Self::on_command_client,
            Arc::clone(&state),
        );

        Self::run_listeners(&state, listeners)?;

        thread::sleep(Duration::MAX);

        Ok(())
    }

    fn on_command_client(
        connection: Box<dyn Connection + Send + 'static>,
        state: Arc<Mutex<Node>>,
    ) {
        thread::spawn(move || {
            let mut connection = connection;
            loop {
                match read(&mut connection) {
                    Ok(packet) => {
                        let result = match packet {
                            Packets::GetConnections => write(
                                &mut connection,
                                Packets::UpdateConnections(state.lock().unwrap().get_clients()),
                            ),
                            Packets::GetRoutes => write(
                                &mut connection,
                                Packets::UpdateRoutes(state.lock().unwrap().get_routes()),
                            ),
                            _ => {
                                error!("Unsupported packet: {:?}", packet);

                                Ok(())
                            }
                        };

                        if let Err(e) = result {
                            error!("Got error: {}", e);
                        }
                    }
                    Err(e) => {
                        if !connection.is_alive() {
                            warn!("Connection {} disconnected!", connection.get_info());
                            break;
                        } else {
                            error!("Got error: {}", e);
                        }
                    }
                }
            }
        });
    }

    fn on_listener_client(
        connection: Box<dyn Connection + Send + 'static>,
        state: Arc<Mutex<Node>>,
    ) {
        thread::spawn(move || {
            let mut connection = connection;
            let mut s = state.lock().unwrap();
            let index = s.clients.len();

            let uuid = Uuid::new_v4().to_string(); //TODO: Calling an OS RNG can pose a problem for security;

            s.clients.push(Client {
                uuid: uuid.clone(),
                connection: connection.try_clone().unwrap(),
                route: vec![uuid],
            });

            write(
                &mut connection,
                Packets::OnClientConnect {
                    id: s.clients.last().unwrap().uuid.clone(),
                    route: s.clients.last().unwrap().route.clone(),
                },
            )
            .unwrap();

            std::mem::drop(s);

            // let is_root = s.parent.is_none();

            loop {
                match read(&mut connection) {
                    Ok(packet) => {
                        let result = match packet {
                            Packets::GetConnections => write(
                                &mut connection,
                                Packets::UpdateConnections(state.lock().unwrap().get_clients()),
                            ),
                            Packets::GetRoutes => write(
                                &mut connection,
                                Packets::UpdateRoutes(state.lock().unwrap().get_routes()),
                            ),
                            Packets::OnClientConnect { id, route } => Ok(()),
                            _ => {
                                error!("Unsupported packet: {:?}", packet);

                                Ok(())
                            }
                        };

                        if let Err(e) = result {
                            error!("Got error: {}", e);
                        }
                    }
                    Err(e) => {
                        if !connection.is_alive() {
                            (&mut state.lock().unwrap()).clients.remove(index);
                            warn!("Connection {} Disconnected!", connection.get_info());
                            break;
                        }

                        error!("Got error: {}", e);
                    }
                }
            }
        });
    }

    fn get_clients(&self) -> Vec<String> {
        self.clients
            .iter()
            .map(|c| format!("Client {}", c.get_info()))
            .collect()
    }

    fn get_routes(&mut self) -> Vec<String> {
        let mut routes = Vec::new();

        for client in &mut self.clients {
            let prefix = client.connection.get_info();

            routes.push(prefix.clone());

            if let Err(e) = write(&mut client.connection, Packets::GetRoutes) {
                error!("Failed to send packet: {}", e);
            }

            if let Ok(Packets::UpdateRoutes(new_routes)) = read(&mut client.connection) {
                routes.append(
                    new_routes
                        .iter()
                        .map(|c| format!("{} -> {}", prefix, c))
                        .collect::<Vec<String>>()
                        .as_mut(),
                );
            }
        }

        routes

        // self.clients
        //     .iter()
        //     .map(|c| format!("Client {}", c.get_info()))
        //     .collect()
    }
}
