use futures::{stream, StreamExt}; // 0.3.8use reqwest::Response;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};
use structopt::StructOpt;
use tokio::time::Duration;

use crate::GlobalArgs;

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "rustbuster-dir-plugin",
    author = "Drew Parker",
    about = "rustbuster dir enumerator"
)]
struct Args {
    #[structopt(default_value = "1", long = "time", help = "request timeout")]
    timeout: u64,
    #[structopt(
        short = "l",
        long = "--show-len",
        help = "displays the size of response"
    )]
    showlen: bool,
    #[structopt(
        short = "r",
        long = "--show-redir",
        help = "displays result of a 301 redirect"
    )]
    showredir: bool,
    #[structopt(
        short = "n",
        long = "--disable-status",
        help = "dont display HTTP status codes"
    )]
    nostatus: bool,
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
    #[structopt(short = "u", long = "url", help = "Target URL")]
    url: String,
}

fn count_lines<R: io::Read>(handle: R) -> Result<u64, io::Error> {
    let mut reader = BufReader::new(handle);
    let mut count = 0;
    let mut line: Vec<u8> = Vec::new();
    while match reader.read_until(b'\n', &mut line) {
        Ok(n) if n > 0 => true,
        Err(e) => return Err(e),
        _ => false,
    } {
        if *line.last().unwrap() == b'\n' {
            count += 1;
        };
    }
    Ok(count)
}

pub async fn exec(gargs: GlobalArgs, mode_args: Vec<String>) -> io::Result<()> {
    let args = Args::from_iter(mode_args);

    if args.wordlist.is_none() {
        eprintln!("[!] Dir module requires global wordlist: -w <path>");
        std::process::exit(-1);
    }

    let wordlist = args.wordlist.unwrap();
    let url = args.url;
    let reader = BufReader::new(File::open(&wordlist)?);
    let word_count: u64 = count_lines(std::fs::File::open(&wordlist).unwrap())?;

    let discard: [u16; 1] = [404];

    if !args.quiet {
        println!(
            "{:-^width$}\n[-] Mode:\tdir\n[-] URL:\t{}\n[-] Wordlist:\t{}\n[-] Count:\t{}\n[-] Threads:\t{}\n[-] Discard:\t{:?}\n{:-^width$}\n",
            "",
            url,
            wordlist.to_str().unwrap(),
            word_count,
            gargs.threads,
            discard,
            "",
            width = 40,
        );
    }

    let pb = if args.noprog {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(word_count)
    };
    pb.set_draw_delta(word_count / 100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {wide_bar:orange/white} {pos:>7}/{len:7} {msg}")
            .progress_chars("##-"),
    );

    let rp = reqwest::redirect::Policy::none();

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .user_agent("Mozilla/5.0")
        .timeout(Duration::from_secs(args.timeout))
        .redirect(rp)
        .build()
        .unwrap();

    let responses = stream::iter(reader.lines())
        .map(|guess| {
            let mut g = String::new();
            g.clone_from(&guess.unwrap());

            let client = client.clone();
            let form_url = if url.ends_with('/') {
                format!("{}{}", url, g)
            } else {
                format!("{}/{}", url, g)
            };

            tokio::spawn(async move {
                let resp = client.get(form_url).send().await;
                //let bytes = resp.unwrap().bytes().await;
                (g, resp)
            })
        })
        .buffer_unordered(gargs.threads);

    responses
        .for_each(|b| async {
            pb.inc(1);
            match b {
                Ok((g, Ok(r))) => {
                    let status = r.status().as_u16();
                    let url = r.url().to_string();
                    let bytes = r.bytes().await;
                    let len = match bytes {
                        Ok(b) => b.len(),
                        _ => 0,
                    };

                    let mut res_str = String::new();

                    // use good res to continue formating based on arguments
                    let good_res = if !discard.contains(&status) && !args.verbose {
                        res_str.push_str(&format!("/{:20}", &g));
                        true
                    } else if args.verbose {
                        if discard.contains(&status) {
                            res_str.push_str(&format!("Drop: /{:20}", &g));
                            true
                        } else {
                            res_str.push_str(&format!("Keep: /{:20}", &g));
                            true
                        }
                    } else {
                        false
                    };

                    if good_res {
                        if !args.nostatus {
                            res_str.push_str(&format!(" ({})", status));
                        }

                        if args.showlen {
                            res_str.push_str(&format!(" [{}]", len));
                        }

                        if status == 301 && args.showredir {
                            res_str.push_str(&format!(" => {}", url));
                        }
                    }

                    if !res_str.is_empty() {
                        if args.noprog {
                            println!("{}", res_str);
                        } else {
                            pb.println(res_str);
                        }
                    }
                }

                Ok((_, Err(e))) => {
                    if args.verbose {
                        eprintln!("[!] {}", e);
                    }
                }
                Err(e) => eprintln!("[!] tokio::JoinError: {}", e),
            }
        })
        .await;

    pb.finish_with_message("[-] Enumeration complete");

    Ok(())
}
