use anyhow::Result;
use clap::Parser;
use cursive::{
    views::{LinearLayout, TextContent, TextView},
    Cursive, CursiveExt, view::{Resizable}, event::{Event, Key},
};
use rust_multi_command::{
    command::{CommandList, RunConfig},
    git::ExecutionReference, ui::KeyHandlerView,
};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::{fs, io::BufReader};
use tokio::{task::JoinHandle, time::Instant};
use uuid::Uuid;

struct StdoutContent(TextContent);

impl Write for StdoutContent {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = String::from_utf8_lossy(buf).to_string();
        self.0.set_content(s);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = RunConfig::parse();

    let mut siv = Cursive::default();
    // siv.menubar().clear();
    // siv.set_window_title("Tsuki Shortcuts");
    
    // let mut layout = LinearLayout::horizontal();

    // let text_content = Arc::new(Mutex::new(TextContent::new("")));
    // let text_view = TextView::new_with_content(text_content.lock().as_deref().unwrap().clone());

    let mut select_view = KeyHandlerView::new();

    // layout.add_child(text_view.full_screen());
    
    // siv.add_fullscreen_layer(layout);
    siv.add_layer(select_view);

    // if let Some(config_file) = config.test_source {
    //     let file = fs::File::open(config_file).expect("Should be able to read the file.");
    //     let file_reader = BufReader::new(file);
    //     let config: CommandList =
    //         serde_json::from_reader(file_reader).expect("Should be able to parse the JSON format.");

    //     run_tests("".to_string(), config, text_content).await?;
    // }

    // siv.set_fps(30);
    siv.run();
    
    Ok(())
}

async fn run_tests(
    _which: String,
    test_config: CommandList,
    text_content: Arc<Mutex<TextContent>>,
) -> Result<()> {
    let _task_list: Vec<JoinHandle<Result<()>>> = test_config
        .into_iter()
        .map(|test_case| {
            let out = text_content.clone();

            tokio::spawn(async move {
                let target_folder = Uuid::new_v4().to_string();

                let repo = ExecutionReference::new(&target_folder, test_case);
                repo.host(&out).await?;
                repo.clone(&out).await?;
                repo.prepare(&out).await?;

                let start_time = Instant::now();
                repo.execute(&out).await?;
                let _elapsed_time = start_time.elapsed().as_millis();

                repo.cleanup(&out).await?;

                Ok(())
            })
        })
        .collect();

    Ok(())
}
