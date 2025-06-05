use std::error::Error;

use clap::{Parser, Subcommand};
use log::trace;
use slint::{ModelRc, VecModel};
use unshell_rs_lib::{
    config::listeners::ListenerConfig,
    listeners::Listener,
    networkers::{ServerTrait, TCPServer},
};

/// The default port that this program looks for
pub static DEFAULT_SERVICE_PORT: u16 = 13370;
/// The default website port that this program looks for
pub static DEFAULT_WEB_PORT: u16 = 8082;

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
    Serve {
        /// Only listen for command clients locally
        #[arg(short, long, default_value_t = false)]
        local: bool,

        /// Port listen to for command clients
        #[arg(short, long, default_value_t = DEFAULT_SERVICE_PORT)]
        service_port: u16,
        // /// Port to listen for website traffic (0 is disabled)
        // #[arg(short, long, default_value_t = DEFAULT_SERVICE_PORT)]
        // web_port: u16,
    },
    Gui {
        /// Listen for command clients remotely aswell
        #[arg(short, long, default_value_t = true)]
        remote: bool,

        /// Port listen to for command clients
        #[arg(short, long, default_value_t = DEFAULT_SERVICE_PORT)]
        service_port: u16,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    let args = Args::parse();

    match args.command {
        Commands::Gui {
            remote,
            service_port,
        } => {}
        Commands::Serve {
            local,
            service_port,
            // web_port,
        } => {}
    }

    // let mut server = Listener::new(TCPServer::bind("0.0.0.0:3000")?);

    // server.run_listener()?;

    Ok(())
}
