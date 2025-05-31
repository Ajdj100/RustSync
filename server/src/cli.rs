use clap::{Parser};

#[derive(Parser)]
#[command(name = "RustSync Server")]
#[command(about = "File backup server", long_about = None)]
pub struct Args {
    //server address
    #[arg(long, short = 'a')]
    pub address: Option<String>,

    //backup location
    #[arg(long = "backup-dir", short = 'd')]
    pub backup_dir: Option<String>,

    //save as config
    #[arg(long)]
    pub save: bool,
}