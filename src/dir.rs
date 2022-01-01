use structopt::StructOpt;
use threadpool::ThreadPool;

use crate::GlobalArgs;

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "rustbuster-dir-plugin",
    author = "Drew Parker",
    about = "rustbuster dir enumerator"
)]
struct Args {
    #[structopt(short = "u", long = "url", help = "Target URL")]
    url: String,
}

pub fn exec(gargs: GlobalArgs, mode_args: Vec<String>, tpool: &ThreadPool) {
    let args = Args::from_iter(mode_args);

    if gargs.wordlist.is_none() {
        eprintln!("[!] Dir module requires global wordlist: -w <path>");
        std::process::exit(-1);
    }

    if !gargs.quiet {
        println!("[-] Scanning {}", args.url);
    }
}
