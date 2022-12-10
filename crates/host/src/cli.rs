use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
   /// Path to the configuration file
   #[arg(short, long, default_value="config.toml")]
   pub config: String,
}