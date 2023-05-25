use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap()]
pub struct RunConfig {
    #[clap(short = 's', long = "source")]
    pub test_source: Option<PathBuf>,
    #[clap(short = 'o', long = "output")]
    pub result_target: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct CommandDetail {
    pub prepare: Option<Vec<String>>,
    pub repository: String,
    pub command: String,
}

pub type CommandList = Vec<CommandDetail>;
