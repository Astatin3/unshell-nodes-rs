use crate::{
    Error,
    layers::{Base64Layer, HandshakeLayer, LayerConfig},
    networkers::{Connection, ProtocolLayer},
};

impl Connection for Box<dyn Connection + Send> {
    fn get_info(&self) -> String {
        (**self).get_info()
    }

    fn is_alive(&self) -> bool {
        (**self).is_alive()
    }

    fn read(&mut self) -> Result<String, Error> {
        (**self).read()
    }

    fn write(&mut self, data: &str) -> Result<(), Error> {
        (**self).write(data)
    }
}

pub fn build_client<C>(base_conn: C, layers: Vec<LayerConfig>) -> Result<Box<dyn Connection>, Error>
where
    C: Connection + 'static,
{
    let mut current_conn: Box<dyn Connection + Send> = Box::new(base_conn);

    for layer_config in &layers {
        current_conn = match layer_config {
            LayerConfig::Base64 => Box::new(Base64Layer::new(current_conn)?),
            LayerConfig::Handshake => {
                let mut handshake_layer = HandshakeLayer::new(current_conn)?;
                handshake_layer.initialize_client()?;
                Box::new(handshake_layer)
            }
        };
    }

    Ok(current_conn)
}

pub fn create_server_builder<C>(
    layers: Vec<LayerConfig>,
) -> Result<Box<dyn Fn(C) -> Result<Box<dyn Connection + Send>, Error>>, Error>
where
    C: Connection + 'static,
{
    Ok(Box::new(
        move |base_conn: C| -> Result<Box<dyn Connection + Send>, Error> {
            let mut current_conn: Box<dyn Connection + Send> = Box::new(base_conn);

            for layer_config in &layers {
                current_conn = match layer_config {
                    LayerConfig::Base64 => Box::new(Base64Layer::new(current_conn)?),
                    LayerConfig::Handshake => {
                        let mut handshake_layer = HandshakeLayer::new(current_conn)?;
                        handshake_layer.initialize_server()?;
                        Box::new(handshake_layer)
                    }
                };
            }

            Ok(current_conn)
        },
    ))
}
