
use std::error::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, watch};

pub struct ChatServer {
    bind: String,
}

impl ChatServer {
    pub fn new(bind: String) -> Self {
        Self { bind }
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(&self.bind).await?;
        println!("Сервер слушает {}", self.bind);

        let (tx, _rx) = broadcast::channel::<String>(100);

        loop {
            let (socket, addr) = listener.accept().await?;
            println!("Подключился клиент {}", addr);

            let tx = tx.clone();
            let mut rx = tx.subscribe();

            tokio::spawn(async move {
                let (read_half, mut write_half) = socket.into_split();
                let mut reader = BufReader::new(read_half);
                let mut line = String::new();

                // Координируем корректное завершение задачи записи.
                let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

                let write_task = tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            _ = shutdown_rx.changed() => {
                                break;
                            }
                            recv = rx.recv() => {
                                let msg = match recv {
                                    Ok(msg) => msg,
                                    Err(_) => break,
                                };
                                if write_half.write_all(msg.as_bytes()).await.is_err() {
                                    println!("Ошибка записи клиенту {}", addr);
                                    break;
                                }
                                if write_half.write_all(b"\n").await.is_err() {
                                    println!("Ошибка записи клиенту {}", addr);
                                    break;
                                }
                            }
                        }
                    }
                });

                // Читаем ввод клиента и рассылаем построчно.
                loop {
                    line.clear();
                    let bytes = match reader.read_line(&mut line).await {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(_) => {
                            println!("Ошибка чтения от клиента {}", addr);
                            break;
                        }
                    };

                    if bytes == 0 {
                        break;
                    }

                    let msg = line.trim_end_matches('\n').to_string();
                    let _ = tx.send(msg);
                }

                let _ = shutdown_tx.send(true);
                let _ = write_task.await;
                println!("Клиент отключился {}", addr);
            });
        }
    }
}
