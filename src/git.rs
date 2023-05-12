use anyhow::{Result, Context};
use std::{fs, env};
use std::path::{PathBuf, Path};
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt};

pub struct RepositoryReference {
    pub folder: String,
    pub path: PathBuf,
    pub url: String

}

impl RepositoryReference {
    pub fn new(folder: &str, url: &str) -> Self {
        let cwd = env::current_dir().unwrap();
        
        Self {
            folder: folder.to_string(),
            path: Path::join(&cwd, &folder),
            url: url.to_string()
        }
    }

    pub async fn prepare(&self) -> Result<()> {
        fs::create_dir(&self.folder).with_context(|| format!("Failed to create directory {:?}", self.path))?;

        println!("Folder created '{:?}'.", self.path);
        Ok(())
    }

    pub async fn clone(&self) -> Result<()> {
        let mut command = Command::new("git")
            .stdout(Stdio::piped())
            .args(["clone", &self.url, "./"])
            .current_dir(&self.path)
            .spawn()
            .with_context(|| format!("Failed to clone repository '{:?}'", &self.url))
            .unwrap();

        let stdout = command.stdout.take().unwrap();
        let mut lines = BufReader::new(stdout).lines();
        while let Some(line) = lines.next_line().await? {
            println!("->> {}: {}", self.url, line);
        }

        Ok(())
    }

    pub async fn cleanup(&self) -> Result<()> {
        fs::remove_dir_all(&self.path).with_context(|| format!("Failed to delete folder '{:?}", self.path))?;
        Ok(())
    }
}