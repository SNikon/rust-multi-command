use anyhow::{Result, Context};
use std::fs;
use std::path::PathBuf;
use tokio::process::Command;

pub struct RepositoryReference {
    pub path: PathBuf,
    pub url: String
}

impl RepositoryReference {
    pub async fn prepare(&mut self) -> Result<()> {
        fs::create_dir(&self.path).with_context(|| format!("Failed to create directory {:?}", self.path))?;

        self.path = self.path.canonicalize().unwrap();
        println!("Folder created '{:?}'.", self.path);
        Ok(())
    }

    pub async fn clone(&self) -> Result<()> {
        let clone_result = Command::new("git")
            .args(["clone", &self.url, "./"])
            .current_dir(&self.path)
            .output().await;

        clone_result.with_context(|| format!("Failed to clone repository '{:?}'", &self.url))?;
        Ok(())
    }

    pub async fn cleanup(&self) -> Result<()> {
        fs::remove_dir_all(&self.path).with_context(|| format!("Failed to delete folder '{:?}", self.path))?;
        Ok(())
    }
}