use futures::{stream, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{header, redirect, Client};
use std::{
    io::{self},
    path::PathBuf,
    str::FromStr,
};
use structopt::StructOpt;
use tokio::time::Duration;

use crate::wordlist::Wordlist;
use crate::GlobalArgs;

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "rustbuster-dir-plugin",
    author = "icon",
    about = "rustbuster dir enumerator plugin"
)]
struct Args {
    #[structopt(default_value = "1", long = "time", help = "request timeout (seconds)")]
    timeout: u64,

    #[structopt(short = "l", long = "show-len", help = "displays the size of response")]
    showlen: bool,
    #[structopt(
        short = "r",
        long = "show-redir",
        help = "displays result of a 301 redirect"
    )]
    showredir: bool,
    #[structopt(
        short = "n",
        long = "disable-status",
        help = "dont display HTTP status codes"
    )]
    nostatus: bool,
    #[structopt(short = "v", long = "verbose", help = "enable verbose output")]
    verbose: bool,
    #[structopt(short = "q", long = "quiet", help = "disable normal output")]
    quiet: bool,
    #[structopt(short = "z", long = "noprog", help = "disable all progress output")]
    noprog: bool,
    #[structopt(short = "e", long = "expand", help = "display full url")]
    exapand: bool,
    #[structopt(
        short = "H",
        long = "header",
        help = "custom headers (-H \"H1: V1\" -H \"H2: V2\")"
    )]
    headers: Option<Vec<String>>,
    #[structopt(
        short = "C",
        long = "cookies",
        help = "custom cookies (-C \"C1=V1\" -C \"C2=C2\")"
    )]
    cookies: Option<Vec<String>>,
    #[structopt(
        short = "A",
        long = "agent",
        help = "user agent",
        default_value = "Mozilla/5.0"
    )]
    agent: String,
    #[structopt(short = "p", long = "prepend", help = "prepend wordlist words (csv)")]
    prepend: Option<String>,
    #[structopt(short = "a", long = "append", help = "append wordlist words (csv)")]
    append: Option<String>,
    #[structopt(short = "x", long = "extensions", help = "extensions to search (csv)")]
    extensions: Option<String>,
    #[structopt(
        short = "s",
        long = "swap",
        help = "swap in for entries that contain {SWAP} (csv)"
    )]
    swap: Option<String>,
    #[structopt(
        short = "f",
        long = "add-slash",
        help = "add a trailing / to each request"
    )]
    addslash: bool,
    #[structopt(short = "k", long = "no-tls", help = "ignore invalid tls warnings")]
    ignoretls: bool,
    #[structopt(
        short = "w",
        long = "wordlist",
        help = "path to wordlist",
        parse(from_os_str)
    )]
    wordlist: Option<PathBuf>,
    #[structopt(short = "u", long = "url", help = "target url")]
    url: String,
}

pub async fn exec(gargs: GlobalArgs, mode_args: Vec<String>) -> io::Result<()> {
    let args = Args::from_iter(mode_args);

    if args.wordlist.is_none() {
        eprintln!("[!] Dir module requires global wordlist: -w <path>");
        std::process::exit(-1);
    }

    let wordlist = args.wordlist.unwrap();
    let url = args.url;

    let wl = Wordlist::new(
        &wordlist,
        args.prepend,
        args.append,
        args.swap,
        args.extensions,
    );

    let discard: [u16; 1] = [404];

    if !args.quiet {
        println!(
            concat!(
                "{:-^width$}\n",
                "[*] Mode:\tdir\n",
                "[*] User-Agent\t{:?}\n",
                "[*] URL:\t{}\n",
                "[*] Wordlist:\t{} ({} entries)\n",
                "[*] Count:\t{}\n",
                "[*] Threads:\t{}\n",
                "[*] Discard:\t{:?}\n",
                "[*] Prepend:\t{:?}\n",
                "[*] Append:\t{:?}\n",
                "[*] Swap:\t{:?}\n",
                "[*] Ext:\t{:?}\n",
                "{:-^width$}\n"
            ),
            "",
            args.agent,
            url,
            wordlist.to_str().unwrap(),
            wl.base_count,
            wl.total_count,
            gargs.threads,
            discard,
            wl.prepend,
            wl.append,
            wl.swap,
            wl.extensions,
            "",
            width = 40,
        );
    }

    let pb = if args.noprog {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(wl.total_count as u64)
    };
    pb.enable_steady_tick(1000);
    pb.set_draw_delta(wl.total_count as u64 / 100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {wide_bar:orange/white} {pos:>7}/{len:7} {msg}")
            .progress_chars("##-"),
    );

    // Headers expected like "Header: Value"
    let mut headers = header::HeaderMap::new();
    args.headers.map(|hdrs| {
        for h in hdrs {
            let (key, val) = h.split_once(": ").unwrap_or(("", ""));
            headers.insert(
                header::HeaderName::from_str(key).unwrap(),
                header::HeaderValue::from_str(val).unwrap(),
            );
        }
    });

    // Cookies expected like "key=value;"
    // Just add Cookie header instead of mucking with cookie stores; overkill
    if args.cookies.is_some() {
        let cookie_str = format!("{};", args.cookies.unwrap().join("; "));
        headers.insert(
            "Cookie",
            header::HeaderValue::from_str(&cookie_str).unwrap(),
        );
    }

    let client = Client::builder()
        .danger_accept_invalid_certs(args.ignoretls)
        .user_agent(args.agent)
        .default_headers(headers)
        .timeout(Duration::from_secs(args.timeout))
        .redirect(redirect::Policy::none())
        .build()
        .unwrap();

    let responses = stream::iter(wl.into_iter())
        .map(|guess| {
            let mut g = String::new();
            g.clone_from(&guess);

            let client = client.clone();
            let form_url = if url.ends_with('/') {
                if args.addslash {
                    format!("{}{}/", url, g)
                } else {
                    format!("{}{}", url, g)
                }
            } else {
                if args.addslash {
                    format!("{}/{}/", url, g)
                } else {
                    format!("{}/{}", url, g)
                }
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
                        if !args.exapand {
                            res_str.push_str(&format!("/{}", &g));
                        } else {
                            res_str.push_str(&format!("{}", &url));
                        }
                        true
                    } else if args.verbose {
                        if discard.contains(&status) {
                            if !args.exapand {
                                res_str.push_str(&format!("Drop: /{}", &g));
                            } else {
                                res_str.push_str(&format!("Drop: {}", &url));
                            }
                            true
                        } else {
                            if !args.exapand {
                                res_str.push_str(&format!("Keep: /{}", &g));
                            } else {
                                res_str.push_str(&format!("Keep: {}", &url));
                            }
                            true
                        }
                    } else {
                        false
                    };

                    if good_res {
                        if args.addslash {
                            res_str.push_str("/");
                        }

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
                            println!("{res_str}");
                        } else {
                            pb.println(res_str);
                        }
                    }
                }

                Ok((_, Err(e))) => {
                    if args.verbose {
                        eprintln!("[!] {e}");
                    }
                }
                Err(e) => eprintln!("[!] tokio::JoinError: {e}"),
            }
        })
        .await;

    pb.finish_with_message("[-] Enumeration complete");

    Ok(())
}
