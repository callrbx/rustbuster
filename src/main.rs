use librb::{dir, GlobalArgs, Mode};
use std::io;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = GlobalArgs::from_args();

    let gargs = args.clone();

    match args.mode {
        Mode::Dir(mode_args) => dir::exec(gargs, mode_args).await?,
    };

    Ok(())
}
