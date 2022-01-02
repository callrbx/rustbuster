use std::{
    collections::VecDeque,
    fs::File,
    io::{self, BufRead, BufReader},
};

use reqwest::Response;
use structopt::StructOpt;
use tokio::runtime::Builder;
use tokio::time::Duration;

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

async fn scanner(url: String, guess: String, verbose: bool, noprog: bool) -> Option<Response> {
    let form_url = if url.ends_with('/') {
        format!("{}{}", url, guess)
    } else {
        format!("{}/{}", url, guess)
    };

    if verbose && !noprog {
        println!("[-] Trying: {}", &form_url);
    }

    let rp = reqwest::redirect::Policy::none();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1))
        .redirect(rp)
        .build()
        .unwrap();

    match client.get(&form_url).send().await {
        Ok(res) => {
            if res.status() != 200 {
                return None;
            }
            Some(res)
        }
        Err(_) => None,
    }
}

pub fn exec(gargs: GlobalArgs, mode_args: Vec<String>) -> io::Result<()> {
    let args = Args::from_iter(mode_args);

    if gargs.wordlist.is_none() {
        eprintln!("[!] Dir module requires global wordlist: -w <path>");
        std::process::exit(-1);
    }

    let runtime = Builder::new_multi_thread()
        .worker_threads(gargs.threads)
        .enable_all()
        .build()
        .unwrap();

    let wordlist = gargs.wordlist.unwrap();
    let url = args.url;

    let reader = BufReader::new(File::open(&wordlist)?);

    if !gargs.quiet {
        println!("[-] Enumerating {} with {:?}", url, wordlist);
    }

    let mut handles = VecDeque::new();
    let mut found = Vec::new();

    for g in reader.lines() {
        match g {
            Ok(guess) => {
                let turl = url.clone();
                let tv = gargs.verbose;
                let tnp = gargs.noprog;
                handles.push_back(runtime.spawn(scanner(turl, guess, tv, tnp)));
            }
            Err(_) => break,
        }
        if handles.len() == handles.capacity() {
            while !handles.is_empty() {
                let x = runtime.block_on(handles.pop_front().unwrap()).unwrap();
                match x {
                    Some(r) => {
                        if !gargs.noprog {
                            println!("[+] {} -> {}", r.url(), r.status())
                        }
                        found.push(r.url().clone());
                    }
                    None => {}
                }
            }
        }
    }

    if found.len() > 0 {
        if !gargs.quiet {
            println!("\n[-] Found:");
        }
        for f in found {
            println!("{}", f);
        }
    } else {
        if !gargs.quiet {
            println!("[-] 0 Results")
        }
    }

    Ok(())
}
