use utils::packet::SyncMessage;
use utils::net::{send, recieve};
use std::error::Error;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt};
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
            let message = SyncMessage::MakeDir {
                relative_path: rel_path,
            };
            send(&mut stream, &message.encode()).await?;
        } else {
            let mut file = File::open(path).await?;
            let meta = file.metadata().await?;
            let message = SyncMessage::BeginFile {
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
                let chunk_msg = SyncMessage::FileChunk {
                    data: buffer[..n].to_vec(),
                };
                send(&mut stream, &chunk_msg.encode()).await?;
            }

            send(&mut stream, &SyncMessage::EndFile.encode()).await?;

            let encoded_checksum = recieve(&mut stream).await?;
            let checksum = SyncMessage::decode(&encoded_checksum);

            if let Some(name) = entry.file_name().to_str() {
                println!("{name}");
                // sent_files.push(name.to_string());
            }
        }
    }
    Ok(())
}
