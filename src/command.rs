use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap()]
pub struct CommandConfig {
    #[clap(short = 's', long = "source")]
    pub test_source: Option<PathBuf>,
    #[clap(short = 'o', long = "output")]
    pub result_target: Option<PathBuf>
}

#[derive(Debug, Deserialize)]
pub struct TestDetail {
    pub repository: String
}

#[derive(Debug, Deserialize)]
pub struct TestConfig {
    pub commands: Vec<String>,
    pub tests: Vec<TestDetail>
}
