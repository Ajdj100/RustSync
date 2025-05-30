use tokio::fs;
use tokio::net::TcpListener;

use utils::net::recieve;
use utils::packet::{self, SyncMessage};
use std::error::Error;
use std::io::Write;
use std::vec;
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
        //recieve incoming communications
        let buf = recieve(stream).await?;
        //decode communications
        let msg = SyncMessage::decode(&buf);

        //handle comm types
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
                //ack the completed file (and send checksum eventually)
                utils::net::send(stream, &packet::SyncMessage::EndFileAck { checksum: vec![0u8] }.encode()).await?;
                cur_file = None;
            }
            SyncMessage::EndFileAck { checksum: _ } => {
                println!("Somehow the server recieved an EndFileAck, this should never happen.")
            }
        }
    }
    Ok(())
}
