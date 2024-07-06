use futures_util::{
    future::{BoxFuture, OptionFuture},
    FutureExt,
};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::time::Instant;

#[derive(Clone)]
pub struct SpeedLimiter(Arc<SpeedLimiterInner>);

struct SpeedLimiterInner {
    byte_count_per: AtomicUsize,
    cur_read: AtomicUsize,
    last_instant: tokio::sync::Mutex<Instant>,
}

const LIMIT_INTERVAL: u64 = 1000;

impl SpeedLimiter {
    // 0 表示不限速
    pub fn new(byte_count_per: Option<usize>) -> Self {
        Self(Arc::new(SpeedLimiterInner {
            byte_count_per: AtomicUsize::new(byte_count_per.unwrap_or(0)),
            cur_read: Default::default(),
            last_instant: tokio::sync::Mutex::new(Instant::now()),
        }))
    }

    pub fn receive_len(&self, len: usize) -> OptionFuture<BoxFuture<()>> {
        let byte_count_per = self.0.byte_count_per.load(Ordering::Relaxed);
        // 0 表示不限速
        if byte_count_per == 0 {
            return None.into();
        }
        let cur_read = self.0.cur_read.fetch_add(len, Ordering::SeqCst);

        if cur_read < byte_count_per {
            return None.into();
        }

        Some(
            async move {
                let mut last_instant = self.0.last_instant.lock().await;
                if self.0.cur_read.load(Ordering::SeqCst) < byte_count_per {
                    return;
                }
                let elapsed_millis = last_instant.elapsed();
                if elapsed_millis.as_millis() < LIMIT_INTERVAL as u128 {
                    let duration = Duration::from_millis(LIMIT_INTERVAL) - elapsed_millis;
                    tokio::time::sleep(duration).await;
                }
                *last_instant = Instant::now();
                self.0.cur_read.fetch_sub(byte_count_per, Ordering::SeqCst);
            }
            .boxed(),
        )
        .into()
    }

    pub(crate) async fn change(&self, byte_count_per: Option<usize>) {
        self.0
            .byte_count_per
            .store(byte_count_per.unwrap_or(0), Ordering::Relaxed);
        self.reset().await;
    }

    pub(crate) async fn reset(&self) {
        let mut last_instant = self.0.last_instant.lock().await;
        self.0.cur_read.store(0, Ordering::Relaxed);
        *last_instant = Instant::now();
    }
}
