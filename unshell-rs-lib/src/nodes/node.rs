use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::Duration,
};

use bincode::{Decode, Encode};
// use std:::{Receiver, Sender};
#[allow(deprecated)]
use rand::{seq::IndexedRandom, thread_rng};

use crate::{
    Error,
    layers::build_client,
    networkers::{ClientTrait, Connection, ServerTrait, TCPClient, TCPServer, run_listener_state},
    nodes::{
        listener::ConnectionConfig,
        packets::{Packets, decode_vec, encode_vec},
    },
};

fn read(c: &mut Box<dyn Connection + Send>) -> Result<Packets, Error> {
    Packets::decode(c.read()?.as_slice())
}

fn write(c: &mut Box<dyn Connection + Send>, packet: Packets) -> Result<(), Error> {
    c.write(&(packet.encode()?))
}

pub struct Node<P>
where
    P: Encode + Decode<()> + Debug + Clone + 'static,
{
    pub state: Arc<Mutex<NodeState<P>>>,
    pub rx: Receiver<(String, P)>,
}

impl<P> Node<P>
where
    P: Encode + Decode<()> + Debug + Clone + Send + 'static,
{
    pub fn run_node(
        id: String,
        clients: Vec<ConnectionConfig>,
        listeners: Vec<ConnectionConfig>,
    ) -> Result<Self, Error>
    where
        P: Encode + Decode<()> + Debug + Clone + 'static,
    {
        // let mut parent = build_client(TCPClient::connect(&parent.socket)?, parent.layers)?;

        let (tx, rx) = mpsc::channel();

        let state = Arc::new(Mutex::new(NodeState::<P> {
            id: id, //Uuid::new_v4().to_string(), //TODO: Calling an OS RNG can pose a problem for security;
            connections: HashMap::new(),
            map: HashMap::new(),
            packet_listener: tx,
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
                        error!("Could not connect to server; {:?}", e);
                    }

                    thread::sleep(Duration::from_millis(1000));
                }
            });
        }

        Ok(Self { state, rx })
    }

    fn run_client(client: ConnectionConfig, state: &Arc<Mutex<NodeState<P>>>) -> Result<(), Error> {
        Self::run_connection(
            build_client(TCPClient::connect(&client.socket)?, client.layers)?,
            state,
        )?;

        Ok(())
    }

    fn on_listener_client(
        connection: Box<dyn Connection + Send + 'static>,
        state: Arc<Mutex<NodeState<P>>>,
    ) {
        thread::spawn(move || {
            if let Err(e) = Self::run_connection(connection, &state) {
                error!("Could not connect; {}", e);
            }
        });
    }

    fn run_connection(
        connection: Box<dyn Connection + Send + 'static>,
        state: &Arc<Mutex<NodeState<P>>>,
    ) -> Result<(), Error> {
        let mut connection = connection;
        let s = state.lock().unwrap();

        let this_uuid = s.id.clone();
        std::mem::drop(s);

        // Send UUID to new connection
        write(&mut connection, Packets::SyncUUID(this_uuid.clone()))?;

        // Recieve UUID
        let uuid_result = read(&mut connection)?;
        let other_uuid = if let Packets::SyncUUID(source) = uuid_result {
            source
        } else {
            return Err(format!("Could not get UUID! Got {:?}", uuid_result).into());
        };

        if (&mut state.lock().unwrap()).knows_client(&other_uuid) {
            write(&mut connection, Packets::ErrorNameExists)?;
            return Err(format!(
                "Attempted to accept connection from node {} which already exists!",
                other_uuid
            )
            .into());
        }

        info!("New Node! {} (direct)", other_uuid);

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
                            Packets::DataUnrouted {
                                src: source,
                                dest,
                                data,
                            } => (&mut state.lock().unwrap()).route_packet(source, dest, data),
                            _ => {
                                error!("Unsupported packet: {:?}", packet);

                                Ok(())
                            }
                        };

                    if let Err(e) = result {
                        error!("Could not parse; {}", e);
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

                    error!("Could not read; {}", e);
                }
            }
        }

        Ok(())
    }
}

pub struct NodeState<P>
where
    P: Encode + Decode<()> + Debug + Clone + 'static,
{
    id: String,
    connections: HashMap<String, Box<dyn Connection + Send>>,
    map: HashMap<String, Vec<String>>,
    packet_listener: Sender<(String, P)>,
}

impl<P> NodeState<P>
where
    P: Encode + Decode<()> + Debug + Clone + Send + 'static,
{
    // Get list of all nodes in map
    fn get_known_nodes(&self) -> Vec<String> {
        self.map.keys().map(|k| k.clone()).collect::<Vec<String>>()
    }

    // Get list of node UUIDs that are directly connected to this node
    fn get_direct_nodes(&self) -> Vec<String> {
        self.connections
            .keys()
            .map(|k| k.clone())
            .collect::<Vec<String>>()
    }

    fn knows_client(&self, id: &String) -> bool {
        self.get_all_nodes().contains(id)
    }

    // Remove all nodes where the routes are empty
    fn remove_null_nodes(&mut self) {
        self.map.retain(|_, routes| !routes.is_empty());
    }

    // Send packet to all directly connected nodes, except maybe one
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

    // Get list of nodes to send to another as known routes
    fn get_routes_to(&self, recv_uuid: &String) -> Vec<String> {
        let mut tx_routes: Vec<String> = Vec::new();

        // Append
        for (map_uuid, routes) in self.map.iter() {
            // Do not transmit a route, which bounces directly back to the sender
            if routes.len() == 1 && &routes[0] == recv_uuid {
                continue;
            }

            tx_routes.push(map_uuid.clone());
        }

        // Append directly connected nodes
        tx_routes.append(&mut self.get_direct_nodes());

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
            if self.get_direct_nodes().contains(&remove_uuid) {
                resend_table = true;
                continue;
            }

            // Check if client still exists, or if it was a direct connection
            // Prevents infinite network loops
            if direct || self.knows_client(&remove_uuid) {
                self.map.remove(&remove_uuid);
                remove_uuids.push(remove_uuid.clone());

                info!(
                    "Node disconnected! {} ({})",
                    remove_uuid,
                    if direct { "direct" } else { "indirect" }
                );

                for (uuid, route) in self.map.iter_mut() {
                    if route.contains(&remove_uuid) {
                        let index = route.iter().position(|r| r == &remove_uuid).unwrap();
                        route.remove(index);
                        remove_uuids.push(uuid.clone());
                    }
                }

                self.remove_null_nodes();
            }
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

        // self.print_map();
    }

    fn extend_routes(&mut self, src: String, routes: Vec<String>) {
        let mut updated = false;

        // Quick sanity check
        if !self.get_direct_nodes().contains(&src) {
            return;
        }

        // Loop through all of the routes in the new recieved route map
        for route in routes {
            // If the route loops back to self, disregard.
            if route == self.id {
                continue;
            }

            // If the connection is already established directly, disregard
            if self.get_direct_nodes().contains(&route) {
                continue;
            }

            // If there is already an entry created for this route
            if self.map.contains_key(&route) {
                // If the route does not already contain the new one
                if !self.map.get(&route).unwrap().contains(&src) {
                    // If the neighbor can be acessed directly, disregard
                    self.map.get_mut(&route).unwrap().push(src.clone());
                    info!("Node update: {} (indirect)", src);
                    updated = true;
                } else {
                    // Else, do nothing
                    continue;
                }
            } else {
                // Else, create the new route entry
                self.map.insert(route.clone(), vec![src.clone()]);
                info!("Node update: {} (indirect)", src);
                updated = true;
            }
        }

        // Solves the case that if a remote node has said that a neighbor has connected before itself has
        let direct_connections = self.get_direct_nodes();
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
        // self.print_map();
    }

    fn route_packet(&mut self, src: String, dest: String, data: Vec<u8>) -> Result<(), Error> {
        if dest == self.id {
            self.packet_listener.send((src, decode_vec::<P>(&data)?))?;
        } else {
            if self.connections.contains_key(&dest) {
                write(
                    self.connections.get_mut(&dest).unwrap(),
                    Packets::DataUnrouted { src, dest, data },
                )?;
            } else if self.map.contains_key(&dest) {
                #[allow(deprecated)]
                let next_uuid = self
                    .map
                    .get(&dest)
                    .unwrap()
                    .choose(&mut thread_rng())
                    .unwrap()
                    .clone();

                write(
                    self.connections.get_mut(&next_uuid).unwrap(),
                    Packets::DataUnrouted { src, dest, data },
                )?;
            } else {
                error!("Could not find route from {} to {}!", src, dest);
            }
        }

        Ok(())
    }

    pub fn send_unrouted(&mut self, dest: String, data: &P) -> Result<(), Error> {
        self.route_packet(self.id.clone(), dest, encode_vec(data)?)
    }

    pub fn get_all_nodes(&self) -> Vec<String> {
        let mut uuids = self.get_known_nodes();
        uuids.append(&mut self.get_direct_nodes());
        uuids
    }

    #[allow(dead_code)]
    fn print_map(&self) {
        info!("\n\n");
        info!("Local addr: {}", self.id);
        info!("Table: ");
        for (uuid, route) in self.map.iter() {
            info!("{} -> [ {:?} ]", uuid, route);
        }
        info!("Direct: {:?}", self.get_direct_nodes());
    }
}
