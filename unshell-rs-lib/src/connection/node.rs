use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    Error,
    connection::{listener::ConnectionConfig, packets::Packets},
    layers::build_client,
    networkers::{ClientTrait, Connection, ServerTrait, TCPClient, TCPServer, run_listener_state},
};

pub struct Node {
    id: String,
    connections: HashMap<String, Box<dyn Connection + Send>>,
    map: HashMap<String, Vec<String>>,
}

fn read(c: &mut Box<dyn Connection + Send>) -> Result<Packets, Error> {
    let a = Packets::decode(c.read()?.as_str());
    info!("Data: {:?}", a);
    a
}

fn write(c: &mut Box<dyn Connection + Send>, packet: Packets) -> Result<(), Error> {
    info!("Wrote: {:?}", packet);
    c.write(packet.encode()?.as_str())
}

impl Node {
    pub fn run_node(
        id: String,
        clients: Vec<ConnectionConfig>,
        listeners: Vec<ConnectionConfig>,
    ) -> Result<(), Error> {
        // let mut parent = build_client(TCPClient::connect(&parent.socket)?, parent.layers)?;

        let state = Arc::new(Mutex::new(Self {
            id: id, //Uuid::new_v4().to_string(), //TODO: Calling an OS RNG can pose a problem for security;
            connections: HashMap::new(),
            map: HashMap::new(),
        }));

        for listener in listeners {
            run_listener_state(
                TCPServer::bind(&listener.socket)?,
                listener.layers,
                Self::on_listener_client,
                Arc::clone(&state),
            );
        }

        for client in clients {
            let state = Arc::clone(&state);
            thread::spawn(move || {
                loop {
                    if let Err(e) = Self::run_client(client.clone(), &state) {
                        error!("{}", e);
                    }

                    thread::sleep(Duration::from_millis(1000));
                }
            });
        }

        thread::sleep(Duration::MAX);

        Ok(())
    }

    fn run_client(client: ConnectionConfig, state: &Arc<Mutex<Node>>) -> Result<(), Error> {
        Self::run_connection(
            build_client(TCPClient::connect(&client.socket)?, client.layers)?,
            state,
        )?;

        Ok(())
    }

    fn on_listener_client(
        connection: Box<dyn Connection + Send + 'static>,
        state: Arc<Mutex<Node>>,
    ) {
        thread::spawn(move || {
            if let Err(e) = Self::run_connection(connection, &state) {
                error!("{}", e);
            }
        });
    }

    fn run_connection(
        connection: Box<dyn Connection + Send + 'static>,
        state: &Arc<Mutex<Node>>,
    ) -> Result<(), Error> {
        let mut connection = connection;
        let s = state.lock().unwrap();

        let this_uuid = s.id.clone();
        std::mem::drop(s);

        // Send UUID to new connection
        write(&mut connection, Packets::SyncUUID(this_uuid.clone()))?;

        // Recieve UUID
        let other_uuid = if let Packets::SyncUUID(source) = read(&mut connection)? {
            source
        } else {
            return Err("Could not get UUID!".into());
        };

        info!("Connection from {} to {}", this_uuid, other_uuid);

        // Add connection
        (&mut state.lock().unwrap())
            .connections
            .insert(other_uuid.clone(), connection.try_clone()?);

        // Update direct connections and the new connections with the new table
        (&mut state.lock().unwrap()).broadcast_table(None);

        loop {
            match read(&mut connection) {
                Ok(packet) => {
                    let result: Result<(), Error> =
                        match packet {
                            Packets::Disconnect { routes } => Ok(
                                (&mut state.lock().unwrap()).disconnect(&other_uuid, routes, false)
                            ),
                            Packets::Update { routes } => Ok((&mut state.lock().unwrap())
                                .extend_routes(other_uuid.clone(), routes)),
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
                        warn!("Connection {} Disconnected!", connection.get_info());
                        let state = &mut state.lock().unwrap();
                        state.connections.remove(&other_uuid);
                        state.disconnect(&this_uuid, vec![other_uuid.clone()], true);

                        break;
                    }

                    error!("Got error: {}", e);
                }
            }
        }

        Ok(())
    }

    fn get_known_clients(&self) -> Vec<String> {
        self.map.keys().map(|k| k.clone()).collect::<Vec<String>>()
    }

    fn get_direct_connections(&self) -> Vec<String> {
        self.connections
            .keys()
            .map(|k| k.clone())
            .collect::<Vec<String>>()
    }

    fn knows_client(&self, id: &String) -> bool {
        self.get_known_clients().contains(id)
    }

    fn remove_null_nodes(&mut self) {
        self.map.retain(|_, routes| !routes.is_empty());
    }

    fn broadcast(&mut self, data: Packets, disclude: Option<&String>) {
        for (uuid, connection) in self.connections.iter_mut() {
            if disclude.is_some() && disclude.unwrap() == uuid {
                continue;
            }
            if let Err(e) = write(connection, data.clone()) {
                error!("Failed to send packet to {}, {}", uuid, e);
            }
        }
    }

    fn get_routes_to(&self, recv_uuid: &String) -> Vec<String> {
        let mut tx_routes: Vec<String> = Vec::new();

        for (map_uuid, routes) in self.map.iter() {
            // Do not transmit a route, which bounces directly back to the sender
            if routes.len() == 1 && &routes[0] == recv_uuid {
                continue;
            }

            tx_routes.push(map_uuid.clone());
        }

        tx_routes.append(&mut self.get_direct_connections());

        tx_routes
    }

    fn broadcast_table(&mut self, disclude: Option<&String>) {
        let packets = self
            .connections
            .iter()
            .map(|(recv_uuid, _)| self.get_routes_to(&recv_uuid))
            .collect::<Vec<Vec<String>>>();

        for (i, (recv_uuid, connection)) in self.connections.iter_mut().enumerate() {
            if let Some(disclude) = disclude {
                if disclude == recv_uuid {
                    continue;
                }
            }

            if let Err(e) = write(
                connection,
                Packets::Update {
                    routes: packets[i].clone(),
                },
            ) {
                error!("Failed to send packet to {}, {}", recv_uuid, e);
            }
        }
    }

    fn disconnect(&mut self, source: &String, routes: Vec<String>, direct: bool) {
        let mut resend_table = false;
        let mut remove_uuids = Vec::new();

        for remove_uuid in routes {
            // Sanity check, in case the current client is still connected
            if self.get_direct_connections().contains(&remove_uuid) {
                resend_table = true;
                continue;
            }

            // Check if client still exists, or if it was a direct connection
            // Prevents infinite network loops
            if direct || self.knows_client(&remove_uuid) {
                self.map.remove(&remove_uuid);
                remove_uuids.push(remove_uuid.clone());

                for (uuid, route) in self.map.iter_mut() {
                    if route.contains(&remove_uuid) {
                        let index = route.iter().position(|r| r == &remove_uuid).unwrap();
                        route.remove(index);
                        remove_uuids.push(uuid.clone());
                    }
                }

                self.remove_null_nodes();
            }

            // for uuid in remove_uuids {
        }

        if !remove_uuids.is_empty() {
            self.broadcast(
                Packets::Disconnect {
                    routes: remove_uuids,
                },
                Some(source),
            );
        }

        if resend_table {
            self.broadcast_table(None);
        }

        // }

        self.print_map();
    }

    fn extend_routes(&mut self, src: String, routes: Vec<String>) {
        let mut updated = false;

        // Quick sanity check
        if !self.get_direct_connections().contains(&src) {
            return;
        }

        // Loop through all of the routes in the new recieved route map
        for route in routes {
            // If the route loops back to self, disregard.
            if route == self.id {
                continue;
            }

            // If the connection is already established directly, disregard
            if self.get_direct_connections().contains(&route) {
                continue;
            }

            // If there is already an entry created for this route
            if self.map.contains_key(&route) {
                // If the route does not already contain the new one
                if !self.map.get(&route).unwrap().contains(&src) {
                    // If the neighbor can be acessed directly, disregard
                    self.map.get_mut(&route).unwrap().push(src.clone());
                    updated = true;
                } else {
                    // Else, do nothing
                    continue;
                }
            } else {
                // Else, create the new route entry
                self.map.insert(route.clone(), vec![src.clone()]);
                updated = true;
            }
        }

        // Solves the case that if a remote node has said that a neighbor has connected before itself has
        let direct_connections = self.get_direct_connections();
        for connection in direct_connections {
            if self.map.contains_key(&connection) {
                self.map.remove(&connection);
            }
        }

        // If something has updated, rebroadcast
        // Prevents infinite network loops
        if updated {
            self.broadcast_table(Some(&src));
        }
        self.print_map();
    }

    fn print_map(&self) {
        info!("\n\n");
        info!("Local addr: {}", self.id);
        info!("Table: ");
        for (uuid, route) in self.map.iter() {
            info!("{} -> [ {:?} ]", uuid, route);
        }
        info!("Direct: {:?}", self.get_direct_connections());
    }
}
