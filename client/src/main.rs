use clap::{Arg, Parser};
use cli::Args;
use config::{load_config, save_config, ClientConfig};
use utils::packet::Packet;
use utils::net::{send, recieve};
use std::error::Error;
use std::path::PathBuf;
use std::process::exit;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use walkdir::WalkDir;

pub mod config;
pub mod cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let config = build_config().unwrap();
    
    let addr = config.remote_addr.unwrap();
    
    let root = std::env::current_dir()?;
    let root_name = root.file_name().unwrap();
    let mut stream = tokio::net::TcpStream::connect(addr).await?;

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


fn build_config() -> Result<ClientConfig, Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut config = load_config().unwrap();

    if let Some(address) = args.address {
        config.remote_addr = Some(address);
    }

    if config.remote_addr.is_none() {
        print!("Server address not specified. Use --address to set it: ");
        exit(1);
    }

    if args.save {
        save_config(&config)?;
        println!("Saved configuration")
    }

    Ok(config)
}
