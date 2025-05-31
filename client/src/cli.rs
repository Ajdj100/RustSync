use clap::{Parser};

#[derive(Parser)]
#[command(name = "RustSync Client")]
#[command(about = "File backup client", long_about = None)]
pub struct Args {
    //server address
    #[arg(long, short = 'a')]
    pub address: Option<String>,

    //save as config
    #[arg(long)]
    pub save: bool,
}