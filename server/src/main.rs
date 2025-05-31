use checksums::hash_file;
use clap::Parser;
use cli::Args;
use config::{load_or_create_config, save_config, ServerConfig};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::{fs::File, path::Path};
use utils::net::recieve;
use utils::packet::Packet;

mod config;
mod cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = build_config().unwrap();

    let addr = config.addr;
    let path: PathBuf = PathBuf::from(&config.backup_dir);

    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {addr}");
    println!("File backups at: {}", path.display());

    loop {
        let (mut socket, _) = listener.accept().await?;
        let path = path.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(&mut socket, &path).await {
                eprintln!("Error handling client: {e:?}");
            }
        });
    }
}

fn build_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut config = load_or_create_config();

    if let Some(address) = args.address {
        config.addr = address;
    }

    if let Some(dir) = args.backup_dir {
        config.backup_dir = dir;
    }

    if args.save {
        save_config(&config)?;
        println!("Saved configuration")
    }

    Ok(config)
}

async fn handle_client(
    stream: &mut tokio::net::TcpStream,
    base_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut cur_file: Option<File> = None;
    let mut file_path: PathBuf = PathBuf::new();

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
                file_path = base_path.join(&relative_path);
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
                //ack the completed file (and send checksum eventually)
                if let Some(file) = &mut cur_file {
                    println!("{}", file_path.display());
                    let hash = hash_file(&file_path, checksums::Algorithm::SHA1);

                    utils::net::send(
                        stream,
                        &Packet::EndFileAck {
                            checksum: hash.into_bytes(),
                        }
                        .encode(),
                    )
                    .await?;
                } else {
                    println!("Couldn't access current file");
                }

                //clean up
                cur_file = None;
                file_path = PathBuf::new();
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
