use std::error::Error;
use std::os::unix::net::UnixStream;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::PathBuf;
use clap::Parser;
use sonitus::msg::Msg;
use sonitus::report::Report;

#[derive(Debug, Parser)]
struct Opts {
    socket: PathBuf,
    #[clap(subcommand)]
    msg: /* wow */ Msg,
}

fn main() -> Result<(), Box<dyn Error>> {
    let Opts { socket, msg } = Opts::parse();
    let line = serde_json::to_string(&msg)?;
    let mut socket = UnixStream::connect(socket)?;
    writeln!(socket, "{}", line)?;
    let mut reader = BufReader::new(socket);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let report = serde_json::from_str::<Report>(&line);
    println!("{:?}", report);
    Ok(())
}

