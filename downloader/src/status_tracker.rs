use std::{path::PathBuf, sync::Arc, time::Duration};

use tokio_util::sync::CancellationToken;

pub struct Status {
    pub total: u64,
    pub downloaded: u64,
    pub current_speed: u64,
    pub downloaded_time: Duration,
    pub average_speed: u64,
}

#[derive(Clone)]
pub struct StatusTracker(Arc<StatusTrackerInner>);

struct StatusTrackerInner {
    path: PathBuf,
    done: CancellationToken,
}

impl StatusTracker {
    pub fn new(path: PathBuf, done: CancellationToken) -> StatusTracker {
        Self(Arc::new(StatusTrackerInner { path, done }))
    }

    pub async fn get_status(&self) -> Status {
        Status {
            total: 22,
            downloaded: 0,
            current_speed: 0,
            downloaded_time: Duration::from_secs(0),
            average_speed: 0,
        }
    }

    pub async fn load_status_from_file(&self) -> Status {
        Status {
            total: 22,
            downloaded: 0,
            current_speed: 0,
            downloaded_time: Duration::from_secs(0),
            average_speed: 0,
        }
    }

    pub async fn save_status_to_file(&self) {
        todo!()
    }
}
