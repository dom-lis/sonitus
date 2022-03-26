use std::sync::{Arc, Mutex};
use std::error::Error;
use std::path::PathBuf;
use clap::Parser;
use tokio::net::{UnixStream, UnixListener};
use rodio::{Sink, OutputStream, OutputStreamHandle};
use sonitus::msg::Msg;
use sonitus::report::Report;

#[derive(Debug, Parser)]
struct Opts {
    socket: PathBuf,
}

struct State {
    stream: OutputStreamHandle,
    sink: Sink,
    queue: Vec<PathBuf>,
    nth: usize,
}

impl State {
    pub fn try_default() -> Result<State, Box<dyn Error>> {
        let (_, stream) = rodio::OutputStream::try_default()?;
        let sink = rodio::Sink::try_new(&stream)?;
        let queue = Vec::new();
        let nth = 0;
        Ok(State { stream, sink, queue, nth })
    }

    pub fn update(&mut self, msg: Msg) -> Report {
        use sonitus::msg::Msg::*;

        match msg {
            Clear => {
                self.sink.stop();
                self.queue.clear();
                self.nth = 0;
                Report::None
            }
            Seek { secs } => {
                Report::None
            }
            Play { path } => {
                self.queue.push(path);
                Report::None
            }
            _ => Report::None,
        }
    }
}

async fn handle_client(mut stream: UnixStream, state: Arc<Mutex<State>>) -> Result<(), Box<dyn Error>> {
    use tokio::io::{BufReader, BufWriter, AsyncBufReadExt, AsyncWriteExt};

    let (read, write) = stream.split();
    let reader = BufReader::new(read);
    let mut writer = BufWriter::new(write);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        let report = match serde_json::from_str::<Msg>(&line) {
            Err(e) => Report::BadMsg(format!("{}", e)),
            Ok(msg) => state.lock().unwrap().update(msg),
        };
        let mut report = serde_json::to_string(&report)?;
        report.push('\n');
        let bytes = report.into_bytes();
        writer.write(&bytes).await?;
        writer.flush().await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let Opts { socket } = Opts::parse();
    let state = State::try_default()?;
    let state = Arc::new(Mutex::from(state));

    let socket = UnixListener::bind(socket)?;
    loop {
        if let Ok((stream, _)) = socket.accept().await {
            tokio::spawn({
                let state = state.clone();
                async move {
                    match handle_client(stream, state).await {
                        Ok(_) => (),
                        Err(e) => println!("{}", e),
                    }
                }
            });
        }
    }
}

