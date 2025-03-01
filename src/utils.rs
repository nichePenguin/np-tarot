use std::{
    path::PathBuf,
    fs,
};

use tokio::time::{Duration, sleep};

pub fn file_watch(
    path: PathBuf,
    period_ms: u64,
    handler: Box<dyn Fn(String) + Sync + Send>) -> tokio::task::JoinHandle<()>
{
    let mut last_hash = String::new(); 
    tokio::spawn(async move { loop {
        sleep(Duration::from_millis(period_ms)).await;
        let new_hash = sha256::try_digest(&path);
        if new_hash.is_err() {
            log::error!("Failed to hash {}", &path.display());
            continue;
        }
        let new_hash = new_hash.unwrap();
        if last_hash != new_hash {
            let file_data = fs::read_to_string(&path);
            if file_data.is_err() {
                log::error!("Failed to read file {}: {}", &path.display(), file_data.err().unwrap());
                continue;
            }
            handler(file_data.unwrap());
            last_hash = new_hash;
            log::debug!("Updated hash for file {} to {}", path.display(), last_hash);
        } else {
            log::trace!("File {} did not change", path.display());
        }
    }})
}


