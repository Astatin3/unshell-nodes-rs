use std::{
    env,
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};

use clap::{Parser, Subcommand};
use log::error;
use unshell_rs::{connect_cli, run_endpoint};

pub static DEFAULT_CONFIG_FILEPATH: &'static str = "server_config.json";

pub static DEFAULT_RELAY_HOST: &'static str = "0.0.0.0";
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
    // Run as a service, and potentially hosting a website
    Relay {
        /// IPv4 to listen for clients on.
        #[arg(short, long, default_value_t = DEFAULT_RELAY_HOST.to_string())]
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
    /// Connect to remote server
    Connect {
        /// Remote server to connect on
        host: String,

        #[arg(short, long, default_value_t = DEFAULT_SERVICE_PORT)]
        /// Port listen to for command clients
        port: u16,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    if env::var("RUST_LOG").is_err() {
        unsafe { env::set_var("RUST_LOG", "info") }
    }

    pretty_env_logger::init();
    let args = Args::parse();

    if let Err(e) = match args.command {
        // Commands::Relay { host, port, .. } => {
        //     let addr = SocketAddr::from_str(format!("{}:{}", host, port).as_str());
        //     if let Err(e) = Node::run() {
        //         error!("{}", e);
        //     }
        // }
        // Commands::Test1 {} => Cli::connect(
        //     "Test1".to_string(),
        //     vec![],
        //     vec![ConnectionConfig {
        //         socket: SocketAddr::from_str("127.0.0.1:13371")?,
        //         layers: vec![],
        //     }],
        // ),
        // Commands::Test2 {} => Cli::connect(
        //     "Test2".to_string(),
        //     vec![ConnectionConfig {
        //         socket: SocketAddr::from_str("127.0.0.1:13371")?,
        //         layers: vec![],
        //     }],
        //     vec![ConnectionConfig {
        //         socket: SocketAddr::from_str("127.0.0.1:13372")?,
        //         layers: vec![],
        //     }],
        // ),
        // Commands::Test3 {} => Cli::connect(
        //     "Test3".to_string(),
        //     vec![ConnectionConfig {
        //         socket: SocketAddr::from_str("127.0.0.1:13372")?,
        //         layers: vec![],
        //     }],
        //     vec![],
        // ), // Commands::Test4 {} => Cli::connect(
        //     "Test4".to_string(),
        //     vec![ConnectionConfig {
        //         socket: SocketAddr::from_str("127.0.0.1:13371")?,
        //         layers: vec![],
        //     }],
        //     vec![ConnectionConfig {
        //         socket: SocketAddr::from_str("127.0.0.1:13374")?,
        //         layers: vec![],
        //     }],
        // ),
        // Commands::Test5 {} => Cli::connect(
        //     "Test5".to_string(),
        //     vec![
        //         ConnectionConfig {
        //             socket: SocketAddr::from_str("127.0.0.1:13372")?,
        //             layers: vec![],
        //         },
        //         ConnectionConfig {
        //             socket: SocketAddr::from_str("127.0.0.1:13374")?,
        //             layers: vec![],
        //         },
        //     ],
        //     vec![ConnectionConfig {
        //         socket: SocketAddr::from_str("127.0.0.1:13375")?,
        //         layers: vec![],
        //     }],
        // ),
        // Commands::Test6 {} => Cli::connect(
        //     "Test6".to_string(),
        //     vec![
        //         ConnectionConfig {
        //             socket: SocketAddr::from_str("127.0.0.1:13373")?,
        //             layers: vec![],
        //         },
        //         ConnectionConfig {
        //             socket: SocketAddr::from_str("127.0.0.1:13375")?,
        //             layers: vec![],
        //         },
        //     ],
        //     vec![],
        // ),
        Commands::Connect { host, port } => {
            let addr = SocketAddr::from_str(format!("{}:{}", host, port).as_str());
            connect_cli(if let Ok(addr) = addr {
                addr
            } else {
                error!("Could not parse address!");
                return Ok(());
            })
        }
        Commands::Relay {
            host,
            port,
            config_filepath: _,
        } => {
            let addr = SocketAddr::from_str(format!("{}:{}", host, port).as_str());
            run_endpoint(if let Ok(addr) = addr {
                addr
            } else {
                error!("Could not parse address!");
                return Ok(());
            })
        }
    } {
        error!("{}", e);
    };

    Ok(())
}
