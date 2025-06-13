use std::{
    io::{Stdin, Stdout, Write},
    net::SocketAddr,
};

use clap::{Parser, Subcommand, command};
use unshell_rs_lib::{
    Error,
    nodes::{ConnectionConfig, NodeContainer},
};

use crate::client::node_cli::NodeCli;

pub trait Cli {
    fn name(&self) -> String;
    fn parse(&mut self, input: Vec<String>) -> Result<(), Error>;
}

#[derive(Debug, Parser)]
pub struct CommandHolder<P>
where
    P: Subcommand,
{
    #[command(subcommand)]
    pub command: P,
}

pub fn connect_cli(socket: SocketAddr) -> Result<(), Error> {
    // let mut client = build_client(TCPClient::connect(&addr)?, vec![])?;

    let node = NodeContainer::connect(
        "Client".to_string(),
        vec![ConnectionConfig {
            socket,
            layers: vec![],
        }],
        vec![],
    )?;

    let mut current_parser = Box::new(NodeCli::new(node)) as Box<dyn Cli>;

    let parse = |current_parser: &mut Box<dyn Cli + 'static>,
                 stdin: &Stdin,
                 stdout: &mut Stdout|
     -> Result<(), Error> {
        let name = current_parser.name();
        print!("Unshell | {}> ", name);
        stdout.flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;

        let input = input.trim();
        if input.is_empty() {
            return Ok(());
        }

        let mut input = split_escape(input);
        // Clap expects the first arg to be the program name
        input.insert(0, name);

        current_parser.parse(input)?;

        Ok(())
    };

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    loop {
        if let Err(e) = parse(&mut current_parser, &stdin, &mut stdout) {
            error!("Failed to parse: {}", e);
        }
    }
}

fn split_escape(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                // Handle escape sequences
                if let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        '\'' | '"' | '\\' | ' ' => {
                            // Escape recognized characters
                            current.push(chars.next().unwrap());
                        }
                        _ => {
                            // For other characters, keep the backslash
                            current.push(ch);
                        }
                    }
                } else {
                    // Backslash at end of string
                    current.push(ch);
                }
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            ' ' if !in_single_quote && !in_double_quote => {
                // Split on unquoted spaces
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
                // Skip consecutive spaces
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    // Add the last token if it exists
    if !current.is_empty() {
        result.push(current);
    }

    result
}
