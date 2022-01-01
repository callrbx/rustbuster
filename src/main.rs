use std::{path::PathBuf, str::FromStr};
use structopt::StructOpt;

#[derive(Debug)]
enum Mode {
    Dir,
}

impl FromStr for Mode {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dir" => Ok(Mode::Dir),
            _ => Err(String::from("Invalid Mode")),
        }
    }

    type Err = String;
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "rustbuster",
    author = "Drew Parker",
    about = "endpoint enumeration tool",
    long_about = "URL/URI enumeration tool written in Rust."
)]
struct Args {
    #[structopt(help = "Mode")]
    mode: Option<Mode>,
}

fn main() {
    println!("Hello, world!");
}
