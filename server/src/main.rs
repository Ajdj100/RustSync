use checksums::hash_file;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

use utils::net::recieve;
use utils::packet::{Packet};
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
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
    let mut file_path: PathBuf =  PathBuf::new();

    loop {
        //recieve incoming communications
        let buf = recieve(stream).await?;
        //decode communications
        let msg = Packet::decode(&buf);

        //handle comm types
        match msg {
            Packet::MakeDir { relative_path } => {
                let path = base_path.join(relative_path);
                fs::create_dir_all(path).await?;
            }
            Packet::BeginFile { relative_path, .. } => {
                let file_path = &base_path.join(relative_path);
                if let Some(parent) = &file_path.parent() {
                    fs::create_dir_all(parent).await?;
                }
                cur_file = Some(File::create(&file_path)?);
            }
            Packet::FileChunk { data } => {
                if let Some(file) = &mut cur_file {
                    file.write_all(&data)?;
                }
            }
            Packet::EndFile => {
                // let hash: String;
                // //ack the completed file (and send checksum eventually)
                // if let Some(file) = &mut cur_file {
                //     println!("{}", file_path.display());
                //     hash = hash_file(&file_path, checksums::Algorithm::SHA1);
                // } else {
                //     hash = String::from("");
                // }
                // utils::net::send(stream, &packet::SyncMessage::EndFileAck { checksum: Vec::from(hash) }.encode()).await?;

                cur_file = None;
            }
            Packet::EndFileAck { checksum: _ } => {
                println!("Somehow the server recieved an EndFileAck, this should never happen.")
            }
            Packet::EndSession => {
                stream.shutdown().await?;
                break;
            }
        }
    }
    Ok(())
}
