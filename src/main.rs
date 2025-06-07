use std::{
    env,
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};

use clap::{Parser, Subcommand};
use log::error;
use unshell_rs::{UnshellClient, UnshellGui, UnshellServer};
// use unshell_rs

pub static DEFAULT_CONFIG_FILEPATH: &'static str = "server_config.json";

// The default port that this program looks for
pub static DEFAULT_SERVICE_PORT: u16 = 13370;
// The default website port that this program looks for
pub static DEFAULT_WEB_PORT: u16 = 8082;

pub static LOCAL_SOCKET: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 12, 34, 56)), 13370);

#[derive(Debug, Parser)]
#[command(name = "unshell-rs")]
#[command(about = "Slick reverse shell tool in rust", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run as a service, and potentially hosting a website
    #[command(arg_required_else_help = true)]
    Server {
        /// IPv4 to listen for clients on.
        host: String,

        /// Port listen to for command clients
        #[arg(short, long, default_value_t = DEFAULT_SERVICE_PORT)]
        port: u16,

        /// Json file to store config
        #[arg(short, long, default_value_t = DEFAULT_CONFIG_FILEPATH.to_string())]
        config_filepath: String,
        // /// Port to listen for website traffic (0 is disabled)
        // #[arg(short, long, default_value_t = DEFAULT_SERVICE_PORT)]
        // web_port: u16,
    },
    /// Run GUI and connect to remote server
    Remote {
        /// Remote server to connect to
        host: String,

        /// Port listen to for command clients
        #[arg(short, long, default_value_t = DEFAULT_SERVICE_PORT)]
        port: u16,
    },
    /// Run both server and GUI on local machine.
    Local {
        /// Json file to store config
        #[arg(short, long, default_value_t = DEFAULT_CONFIG_FILEPATH.to_string())]
        config_filepath: String,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    if env::var("RUST_LOG").is_err() {
        unsafe { env::set_var("RUST_LOG", "info") }
    }

    pretty_env_logger::init();
    let args = Args::parse();

    match args.command {
        Commands::Local { config_filepath } => {
            let mut server = UnshellServer::from_filepath(config_filepath.as_str());
            server.run(LOCAL_SOCKET)?;

            let client = UnshellClient::new(LOCAL_SOCKET)?;

            UnshellGui::start(client)?;
        }
        Commands::Remote { host, port } => {
            let addr = SocketAddr::from_str(format!("{}:{}", host, port).as_str());
            let client = UnshellClient::new(if let Ok(addr) = addr {
                addr
            } else {
                error!("Could not parse address!");
                return Ok(());
            })?;

            UnshellGui::start(client)?;
        }
        Commands::Server {
            host,
            port,
            config_filepath,
        } => {
            let mut unshell_server = UnshellServer::from_filepath(config_filepath.as_str());

            let addr = SocketAddr::from_str(format!("{}:{}", host, port).as_str());
            if let Ok(addr) = addr {
                unshell_server.run(addr)?;
            } else {
                error!("Could not parse address!");
                return Ok(());
            }

            loop {}
        }
    };

    Ok(())
}
