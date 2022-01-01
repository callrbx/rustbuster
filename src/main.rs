use std::path::PathBuf;

use structopt::StructOpt;
use threadpool::ThreadPool;

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
        default_value = "4",
        short = "t",
        long = "threads",
        help = "thread count"
    )]
    threads: usize,
    #[structopt(short = "v", long = "verbose", help = "enable verbose output")]
    verbose: bool,
    #[structopt(short = "q", long = "quiet", help = "disable normal output")]
    quiet: bool,
    #[structopt(short = "z", long = "noprog", help = "disable all progress output")]
    noprog: bool,
    #[structopt(
        short = "w",
        long = "wordlist",
        help = "path to wordlist",
        parse(from_os_str)
    )]
    wordlist: Option<PathBuf>,
    #[structopt(name = "mode", help = "Mode", subcommand)]
    mode: Mode,
}

fn main() {
    let args = GlobalArgs::from_args();

    let tpool = ThreadPool::new(args.threads);

    let gargs = args.clone();

    match args.mode {
        Mode::Dir(mode_args) => dir::exec(gargs, mode_args, &tpool),
    };
}
