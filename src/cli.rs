use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// IP address of the oven
    #[arg(short, long)]
    pub address: String,
}
