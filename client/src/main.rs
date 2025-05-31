use utils::packet::Packet;
use utils::net::{send, recieve};
use std::error::Error;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use walkdir::WalkDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let root = std::env::current_dir()?;
    let root_name = root.file_name().unwrap();
    let mut stream = tokio::net::TcpStream::connect("127.0.0.1:2600").await?;
    let mut sent_files: Vec<String> = Vec::new();

    for entry in WalkDir::new(&root) {
        let entry = entry?;
        let path = entry.path();

        //recreate parent folder
        let rel = path.strip_prefix(&root)?;
        let rel_path = PathBuf::from(&root_name).join(rel);

        if entry.file_type().is_dir() {
            let message = Packet::MakeDir {
                relative_path: rel_path,
            };
            send(&mut stream, &message.encode()).await?;
        } else {
            let mut file = File::open(path).await?;
            let meta = file.metadata().await?;
            let message = Packet::BeginFile {
                relative_path: rel_path.clone(),
                file_size: meta.len(),
            };
            send(&mut stream, &message.encode()).await?;

            let mut buffer = [0u8; 65536];
            loop {
                let n = file.read(&mut buffer).await?;
                if n == 0 {
                    break;
                }
                let chunk_msg = Packet::FileChunk {
                    data: buffer[..n].to_vec(),
                };
                send(&mut stream, &chunk_msg.encode()).await?;
            }

            //end file communications
            send(&mut stream, &Packet::EndFile.encode()).await?;

            let encoded_ack = recieve(&mut stream).await?;
            let ack = Packet::decode(&encoded_ack);

            //compute local checksum
            let client_checksum = checksums::hash_file(&entry.clone().into_path(), checksums::Algorithm::SHA1);

            match ack {
                Packet::EndFileAck { checksum} => {
                    let server_checksum = String::from_utf8_lossy(&checksum);
                    if server_checksum == client_checksum {
                        println!("[OK] {}", rel_path.file_name().unwrap().to_string_lossy());
                    } else {
                        println!("[FAIL] {}", rel_path.file_name().unwrap().to_string_lossy())
                    }
                }
                _ => {
                    println!("Client recieved unexpected packet");
                }
            }
        }
    }
    send(&mut stream, &Packet::EndSession.encode()).await?;
    stream.shutdown().await?;
    println!("done");
    Ok(())
}
