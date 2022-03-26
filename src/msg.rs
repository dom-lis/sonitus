use std::path::PathBuf;
use clap::Subcommand;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Subcommand)]
pub enum Msg {
    /// clear queue (but keep playing current track)
    Clear,

    /// list tracks in queue
    Queue,

    /// pause playback
    Pause,

    /// skip currently playing track
    Skip,

    /// stop playback and clear queue
    Stop,

    /// add file (or directory) to queue
    Play { path: PathBuf },

    /// remove track from queue
    Rm { nth: usize, until: Option<usize> },

    /// seek
    Seek { secs: usize }
}

