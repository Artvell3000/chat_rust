// Описание CLI: подкоманды и параметры запуска.
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "chat", version, about = "TCP chat (client and server)")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Server {
        // Адрес и порт для прослушивания.
        #[arg(long)]
        bind: String,
    },
    Client {
        // Адрес сервера.
        #[arg(long)]
        addr: String,
        // Имя пользователя.
        #[arg(long)]
        name: String,
    },
}
