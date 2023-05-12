use anyhow::{Result};
use git2::{Repository, RemoteCallbacks, build::{CheckoutBuilder, RepoBuilder}, FetchOptions, Progress, Cred};
use std::{cell::RefCell, path::{PathBuf}, io::{self, Write}, str::FromStr};

struct State {
    current: usize,
    path: Option<PathBuf>,
    progress: Option<Progress<'static>>,
    total: usize,
    newline: bool
}

pub struct RepositoryReference {
    pub ssh_key: Option<PathBuf>,
    pub path: PathBuf,
    pub repo: Option<Repository>,
    pub url: String
}

fn print(state: &mut State) {
    let stats = state.progress.as_ref().unwrap();
    let network_pct = (100 * stats.received_objects()) / stats.total_objects();
    let index_pct = (100 * stats.indexed_objects()) / stats.total_objects();
    let co_pct = if state.total > 0 {
        (100 * state.current) / state.total
    } else {
        0
    };
    let kbytes = stats.received_bytes() / 1024;
    if stats.received_objects() == stats.total_objects() {
        if !state.newline {
            println!();
            state.newline = true;
        }
        print!(
            "Resolving deltas {}/{}\r",
            stats.indexed_deltas(),
            stats.total_deltas()
        );
    } else {
        print!(
            "net {:3}% ({:4} kb, {:5}/{:5})  /  idx {:3}% ({:5}/{:5})  \
             /  chk {:3}% ({:4}/{:4}) {}\r",
            network_pct,
            kbytes,
            stats.received_objects(),
            stats.total_objects(),
            index_pct,
            stats.indexed_objects(),
            stats.total_objects(),
            co_pct,
            state.current,
            state.total,
            state
                .path
                .as_ref()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
        )
    }
    io::stdout().flush().unwrap();
}

impl RepositoryReference {
    pub fn clone(&self) -> Result<()> {
        let state = RefCell::new(State {
            progress: None,
            total: 0,
            current: 0,
            path: None,
            newline: false,
        });

        let mut cb = RemoteCallbacks::new();
        cb.transfer_progress(|stats| {
            let mut state = state.borrow_mut();
            state.progress = Some(stats.to_owned());
            print(&mut *state);
            true
        });
    
        let mut co = CheckoutBuilder::new();
        co.progress(|path, cur, total| {
            let mut state = state.borrow_mut();
            state.path = path.map(|p| p.to_path_buf());
            state.current = cur;
            state.total = total;
            print(&mut *state);
        });

        if let Some(key_path) = self.ssh_key.clone() {
            let pvt_key = key_path.canonicalize().unwrap();
            let pub_key = PathBuf::from_str(&format!("{}.pub", pvt_key.to_str().unwrap())).unwrap();

            cb.credentials(move |_url, username_from_url, _allowed_types| {
                println!("->> ssh user {:?}", username_from_url);
                println!("->> ssh pub key {:?}", pub_key);
                println!("->> ssh pvt key {:?}", pvt_key);

                Cred::ssh_key(
                    username_from_url.unwrap(),
                    Some(&pub_key),
                    &pvt_key,
                    None,
                )
            });
        }
    
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);
        RepoBuilder::new()
            .fetch_options(fo)
            .with_checkout(co)
            .clone(&self.url, &self.path)?;
        
        println!();
    
        Ok(())
    }
}