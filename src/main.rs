use anyhow::Result;
use clap::Parser;
use tokio::{time::Instant, process::Command};
use uuid::Uuid;
use std::{fs, io::BufReader, time::Duration, path::{PathBuf}};
use rust_multi_command::{command::{CommandConfig, TestConfig}, git::RepositoryReference};

#[tokio::main]
async fn main() -> Result<()> {
    let config = CommandConfig::parse();
    
    if let Some(config_file) = config.test_source {
        let file = fs::File::open(config_file).expect("Should be able to read the file.");
        let file_reader = BufReader::new(file);
        let config_data: TestConfig = serde_json::from_reader(file_reader).expect("Should be able to parse the JSON format.");

        let task_list: Vec<_> = config_data.commands
            .iter()
            .flat_map(|command| config_data.tests
                .iter()
                .map(|test_case| {
                    let mv_command = command.clone();
                    let mv_repository = test_case.repository.clone();
                    
                    tokio::spawn(async move {
                        let target_folder = Uuid::new_v4().to_string();

                        let mut repo = RepositoryReference { path: PathBuf::from(target_folder), url: mv_repository }; 
                        let create_result = repo.prepare().await;

                        if create_result.is_err() {
                            println!("->> {:?}", create_result);
                            return format!("[{:?}] Elapsed {:?}ms on '{:?}' -- '{:?}'.", repo.path, Duration::MAX.as_millis(), mv_command, repo.url)
                        }

                        let clone_result = repo.clone().await;

                        if clone_result.is_err() {
                            println!("->> {:?}", clone_result);
                            let deletion_result = repo.cleanup().await;
                            if deletion_result.is_err() { println!("->> {:?}", deletion_result); }   
                            return format!("[{:?}] Elapsed {:?}ms on '{:?}' -- '{:?}'.", repo.path, Duration::MAX.as_millis(), mv_command, repo.url)
                        }

                        println!("Repository cloned '{:?}' at '{:?}'.", repo.path, repo.url);
                        let start_time = Instant::now();
    
                        let _install_result = Command::new("yarn")
                            .current_dir(&repo.path)
                            .output().await;
    
                        // Do stuff
                        
                        let deletion_result = repo.cleanup().await;
                        if deletion_result.is_err() { println!("->> {:?}", deletion_result); }   

                        return format!("[{:?}] Elapsed {:?}ms on '{:?}' -- '{:?}'.", repo.path, start_time.elapsed().as_millis(), mv_command, repo.url)
                    })
                }))
            .collect();

        for task in task_list {
            let result = task.await.expect("task failed");
            println!("{}", result);
        }
    }

    return Ok(());
}