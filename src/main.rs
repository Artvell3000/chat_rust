mod cli;
mod client;
mod server;

use clap::Parser;
use std::error::Error;

use cli::{Cli, Commands};
use client::ChatClient;
use server::ChatServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Server {
            bind,
        } => {
            // Запуск режима TCP-сервера.
            let server = ChatServer::new(bind);
            server.run().await?;
        }
        Commands::Client { addr, name } => {
            // Запуск режима TCP-клиента.
            let client = ChatClient::new(addr, name);
            client.run().await?;
        }
    }
    Ok(())
}
