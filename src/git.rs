use anyhow::{Context, Result};
use cursive::views::TextContent;
use retry::{delay::Fixed, retry};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::{env, fs};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::command::CommandDetail;

pub struct ExecutionReference {
    execution: CommandDetail,
    folder: String,
    path: PathBuf,
}

impl ExecutionReference {
    pub fn new(folder: &str, command: CommandDetail) -> Self {
        let cwd = env::current_dir().unwrap();

        Self {
            execution: command,
            folder: folder.to_string(),
            path: Path::join(&cwd, folder),
        }
    }

    pub async fn host(&self, data_out: &Arc<Mutex<TextContent>>) -> Result<()> {
        fs::create_dir(&self.folder)
            .with_context(|| format!("Failed to create directory {:?}", self.path))?;

        data_out
            .lock()
            .unwrap()
            .append(format!("Directory {:?} created.\n", self.path));

        Ok(())
    }

    pub async fn clone(&self, data_out: &Arc<Mutex<TextContent>>) -> Result<()> {
        data_out.lock().unwrap().append(format!(
            "Cloning target {:?} command on {:?}.",
            self.execution.repository, self.path
        ));

        let mut command = Command::new("git")
            .args(["clone", &self.execution.repository, "./"])
            .current_dir(&self.path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| {
                format!(
                    "Failed to clone repository '{:?}'",
                    &self.execution.repository
                )
            })
            .unwrap();

        let command_stdout = command
            .stdout
            .take()
            .expect("Internal error, could not take stdout");
        let command_stderr = command
            .stderr
            .take()
            .expect("Internal error, could not take stderr");

        let (stderr_tx, stderr_rx) = mpsc::channel(10);

        let task_data_out = data_out.clone();
        let stdout_thread: JoinHandle<Result<()>> = tokio::spawn(async move {
            let mut stdout_lines = BufReader::new(command_stdout);
            let mut line = String::new();

            while stdout_lines.read_line(&mut line).await.unwrap() > 0 {
                task_data_out.lock().unwrap().append(line.clone());
                line.clear();
            }

            Ok(())
        });

        let stderr_thread: JoinHandle<Result<()>> = tokio::spawn(async move {
            let mut stderr_lines = BufReader::new(command_stderr);
            let mut line = String::new();

            while stderr_lines.read_line(&mut line).await.unwrap() > 0 {
                stderr_tx.send(line.clone()).await?;
                line.clear();
            }

            Ok(())
        });

        let _status = command.wait();

        stdout_thread.await?.unwrap();
        stderr_thread.await?.unwrap();

        Ok(())
    }

    pub async fn prepare(&self, data_out: &Arc<Mutex<TextContent>>) -> Result<()> {
        let (stderr_tx, stderr_rx) = mpsc::channel(10);

        data_out
            .lock()
            .unwrap()
            .append(format!("Preparing command on {:?}.\n", self.path));

        if let Some(prepare) = &self.execution.prepare {
            for prepare_case in prepare.iter() {
                let mut prepare_args: Vec<String> = prepare_case
                    .split_whitespace()
                    .map(|f| f.to_string())
                    .collect();
                let mut command_args: Vec<String> = vec!["/c".to_string()];
                command_args.append(&mut prepare_args);

                let mut command = Command::new("cmd")
                    .args(&command_args)
                    .current_dir(self.folder.clone())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .with_context(|| {
                        format!("Failed to execute prepare command '{:?}'", &command_args)
                    })
                    .unwrap();

                let command_stdout = command
                    .stdout
                    .take()
                    .expect("Internal error, could not take stdout");
                let command_stderr = command
                    .stderr
                    .take()
                    .expect("Internal error, could not take stderr");

                let task_data_out = data_out.clone();
                let stdout_thread: JoinHandle<Result<()>> = tokio::spawn(async move {
                    let mut stdout_lines = BufReader::new(command_stdout);
                    let mut line = String::new();

                    while stdout_lines.read_line(&mut line).await.unwrap() > 0 {
                        task_data_out.lock().unwrap().append(line.clone());
                        line.clear();
                    }

                    Ok(())
                });

                let task_stderr_tx = stderr_tx.clone();
                let stderr_thread: JoinHandle<Result<()>> = tokio::spawn(async move {
                    let mut stderr_lines = BufReader::new(command_stderr);
                    let mut line = String::new();

                    while stderr_lines.read_line(&mut line).await.unwrap() > 0 {
                        task_stderr_tx.send(line.clone()).await?;
                        line.clear();
                    }

                    Ok(())
                });

                let _status = command.wait();

                stdout_thread.await?.unwrap();
                stderr_thread.await?.unwrap();
            }
        }

        Ok(())
    }

    pub async fn execute(&self, _data_out: &Arc<Mutex<TextContent>>) -> Result<()> {
        Ok(())
    }

    pub async fn cleanup(&self, data_out: &Arc<Mutex<TextContent>>) -> Result<()> {
        retry(Fixed::from_millis(1000), || fs::remove_dir_all(&self.path))
            .with_context(|| format!("Failed to delete folder '{:?}.", self.path))?;

        data_out
            .lock()
            .unwrap()
            .append(format!("Directory {:?} deleted.\n", self.path));

        Ok(())
    }
}
