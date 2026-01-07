
use colored::Colorize;
use std::error::Error;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::watch;

#[derive(Clone, Debug, PartialEq, Eq)]
enum ClientExit {
    Running,
    Disconnected,
    SendFailed,
    ReadFailed,
    StdinClosed,
}

pub struct ChatClient {
    addr: String,
    name: String,
}

impl ChatClient {
    pub fn new(addr: String, name: String) -> Self {
        Self { addr, name }
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        let stream = match TcpStream::connect(&self.addr).await {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("Failed to connect to {}: {}", self.addr, err);
                return Err(Box::new(err));
            }
        };

        let (read_half, mut write_half) = stream.into_split();

        // Общее состояние завершения, чтобы чтение/запись могли остановить друг друга.
        let (exit_tx, exit_rx) = watch::channel(ClientExit::Running);

        let name_owned = self.name.clone();
        let mut reader_exit_rx = exit_rx.clone();
        let reader_exit_tx = exit_tx.clone();
        let reader_task = tokio::spawn(async move {
            let mut reader = BufReader::new(read_half);
            let mut line = String::new();
            let mention = format!("@{}", name_owned);

            loop {
                line.clear();
                tokio::select! {
                    _ = reader_exit_rx.changed() => {
                        break;
                    }
                    res = reader.read_line(&mut line) => {
                        match res {
                            Ok(0) => {
                                eprintln!("Disconnected from server.");
                                let _ = reader_exit_tx.send(ClientExit::Disconnected);
                                break;
                            }
                            Ok(_) => {
                                let msg = line.trim_end_matches('\n');
                                // Подсветка упоминаний текущего пользователя.
                                if msg.contains(&mention) {
                                    println!("{}", msg.yellow());
                                } else {
                                    println!("{}", msg);
                                }
                            }
                            Err(err) => {
                                eprintln!("Connection error: {}", err);
                                let _ = reader_exit_tx.send(ClientExit::ReadFailed);
                                break;
                            }
                        }
                    }
                }
            }
        });

        let name_owned = self.name.clone();
        let mut writer_exit_rx = exit_rx.clone();
        let writer_exit_tx = exit_tx.clone();
        let writer_task = tokio::spawn(async move {
            let stdin = io::stdin();
            let mut reader = BufReader::new(stdin);
            let mut line = String::new();

            loop {
                line.clear();
                tokio::select! {
                    _ = writer_exit_rx.changed() => {
                        break;
                    }
                    res = reader.read_line(&mut line) => {
                        let bytes = match res {
                            Ok(0) => {
                                let _ = writer_exit_tx.send(ClientExit::StdinClosed);
                                break;
                            }
                            Ok(n) => n,
                            Err(_) => {
                                let _ = writer_exit_tx.send(ClientExit::StdinClosed);
                                break;
                            }
                        };

                        if bytes == 0 {
                            let _ = writer_exit_tx.send(ClientExit::StdinClosed);
                            break;
                        }

                        let msg = line.trim_end_matches('\n');
                        let formatted = format!("{}: {}", name_owned, msg);
                        // Отправляем одну строку в согласованном формате.
                        if write_half.write_all(formatted.as_bytes()).await.is_err() {
                            eprintln!("Failed to send message.");
                            let _ = writer_exit_tx.send(ClientExit::SendFailed);
                            break;
                        }
                        if write_half.write_all(b"\n").await.is_err() {
                            eprintln!("Failed to send message.");
                            let _ = writer_exit_tx.send(ClientExit::SendFailed);
                            break;
                        }
                    }
                }
            }
        });

        let mut exit_rx_main = exit_rx;
        loop {
            let _ = exit_rx_main.changed().await;
            if *exit_rx_main.borrow() != ClientExit::Running {
                break;
            }
        }

        let _ = reader_task.await;
        let _ = writer_task.await;

        let exit_state = exit_rx_main.borrow().clone();
        match exit_state {
            ClientExit::Running | ClientExit::StdinClosed => Ok(()),
            ClientExit::Disconnected => Err(Box::new(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "Disconnected from server",
            ))),
            ClientExit::SendFailed => Err(Box::new(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Failed to send message",
            ))),
            ClientExit::ReadFailed => Err(Box::new(io::Error::new(
                io::ErrorKind::ConnectionReset,
                "Connection error",
            ))),
        }
    }
}
