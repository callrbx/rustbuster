use structopt::StructOpt;

pub mod dir;

#[derive(Debug, StructOpt, Clone)]
pub enum Mode {
    #[structopt(external_subcommand)]
    Dir(Vec<String>),
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "rustbuster",
    author = "icon",
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
    pub threads: usize,
    #[structopt(name = "mode", help = "Mode", subcommand)]
    pub mode: Mode,
}
