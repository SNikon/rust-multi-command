use anyhow::Result;
use clap::Parser;
use tokio::{time::Instant, process::Command};
use uuid::Uuid;
use std::{fs, io::BufReader, time::Duration, path::Path};
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
                    let mv_ssh_key = config.key_file.clone();
                    
                    tokio::spawn(async move {
                        let target_folder = Uuid::new_v4().to_string();
                        let create_result = fs::create_dir(&target_folder);

                        if create_result.is_err() {
                            println!("Failed to create directory '{:?}'.", target_folder);
                            return format!("[{:?}] Elapsed {:?}ms on '{:?}' -- '{:?}'.", target_folder, Duration::MAX.as_millis(), mv_command, mv_repository)
                        }

                        let target_folder = Path::new(&target_folder).canonicalize().unwrap();
                        println!("Folder created '{:?}'.", target_folder);
                        
                        let repo = RepositoryReference { ssh_key: mv_ssh_key, path: target_folder, url: mv_repository, repo: Option::None };
                        let clone_result = repo.clone();

                        if clone_result.is_err() {
                            println!("Failed to clone repository '{:?}' -- {:?}", repo.url, clone_result);
                            let deletion_result = fs::remove_dir_all(&repo.path);
                            if deletion_result.is_err() {
                                println!("{:?}", deletion_result);
                            }   
                            return format!("[{:?}] Elapsed {:?}ms on '{:?}' -- '{:?}'.", repo.path, Duration::MAX.as_millis(), mv_command, repo.url)
                        }

                        println!("Repository cloned '{:?}' at '{:?}'.", repo.path, repo.url);
                        let start_time = Instant::now();
    
                        let install_result = Command::new("yarn")
                            .current_dir(&repo.path)
                            .output().await;
    
                        println!("{:?}", mv_command);
                        println!("{:?}", repo.url);

                        // println!("{:?}", install_result.try_into();
                        
                        // Do stuff
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