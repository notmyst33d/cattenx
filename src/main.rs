use serde::Deserialize;
use std::env;
use std::error::Error;
use std::sync::Arc;
use tokio::fs;
use tokio::io::{copy_bidirectional, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Deserialize)]
struct Config {
    mtproto: String,
    https: String,
}

async fn client_thread(config: Arc<Config>, mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut id = [0u8; 4];
    socket.read_exact(&mut id).await?;

    let mut connection = if id != *b"HEAD" && id != *b"POST" && id != *b"GET " && id != *b"OPTI" && id != [22, 3, 1, 2] {
        TcpStream::connect(&config.mtproto).await?
    } else {
        TcpStream::connect(&config.https).await?
    };

    connection.write(&id).await?;
    copy_bidirectional(&mut socket, &mut connection).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:443").await?;
    let config: Arc<Config> = Arc::new(toml::from_str(
        &fs::read_to_string(env::var("CONFIG").unwrap_or("config.toml".into())).await?,
    )?);

    loop {
        let (socket, _) = listener.accept().await?;
        let config = config.clone();
        tokio::spawn(async move {
            let _ = client_thread(config, socket).await;
        });
    }
}
