use std::io;
use structopt::StructOpt;

mod dir;

#[derive(Debug, StructOpt, Clone)]
enum Mode {
    #[structopt(external_subcommand)]
    Dir(Vec<String>),
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "rustbuster",
    author = "Drew Parker",
    about = "modular endpoint enumeration tool",
    long_about = "web enumeration tool written in Rust."
)]
pub struct GlobalArgs {
    #[structopt(
        default_value = "10",
        short = "t",
        long = "threads",
        help = "thread count"
    )]
    threads: usize,
    #[structopt(name = "mode", help = "Mode", subcommand)]
    mode: Mode,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = GlobalArgs::from_args();

    let gargs = args.clone();

    match args.mode {
        Mode::Dir(mode_args) => dir::exec(gargs, mode_args).await?,
    };

    Ok(())
}
