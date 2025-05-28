use tokio::fs;
use tokio::{io::AsyncReadExt, net::TcpListener};

use packet::SyncMessage;
use std::error::Error;
use std::io::Write;
use std::{fs::File, path::Path};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:2600".to_string();

    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {addr}");

    let path = Path::new("./backup");

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(e) = handle_client(&mut socket, path).await {
                eprintln!("Error handling client: {e:?}");
            }
        });
    }
}

async fn handle_client(
    stream: &mut tokio::net::TcpStream,
    base_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut cur_file: Option<File> = None;

    loop {
        let mut len_buf = [0u8; 4];
        if stream.read_exact(&mut len_buf).await.is_err() {
            break;
        }
        let len = u32::from_be_bytes(len_buf);

        let mut buf = vec![0u8; len as usize];
        stream.read_exact(&mut buf).await?;

        let msg = packet::SyncMessage::decode(&buf);

        match msg {
            SyncMessage::MakeDir { relative_path } => {
                let path = base_path.join(relative_path);
                fs::create_dir_all(path).await?;
            }
            SyncMessage::BeginFile { relative_path, .. } => {
                let full_path = base_path.join(relative_path);
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent).await?;
                }
                cur_file = Some(File::create(full_path)?);
            }
            SyncMessage::FileChunk { data } => {
                if let Some(file) = &mut cur_file {
                    file.write_all(&data)?;
                }
            }
            SyncMessage::EndFile => {
                cur_file = None;
            }
        }
    }
    Ok(())
}
