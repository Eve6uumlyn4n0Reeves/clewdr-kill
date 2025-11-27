use std::collections::VecDeque;
use std::sync::Mutex;
use tracing::{error, warn};

use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct StatsSample {
    pub timestamp: DateTime<Utc>,
    pub total_requests: u64,
    pub success_rate: f64,
    pub average_response_time: u64,
}

const MAX_SAMPLES: usize = 120;

static HISTORY: Mutex<VecDeque<StatsSample>> = Mutex::new(VecDeque::new());

pub fn record_sample(sample: StatsSample) {
    match HISTORY.lock() {
        Ok(mut history) => {
            history.push_back(sample);
            while history.len() > MAX_SAMPLES {
                history.pop_front();
            }
        }
        Err(e) => {
            error!("Failed to acquire lock for recording stats sample: {:?}", e);
        }
    }
}

pub fn get_samples() -> Vec<StatsSample> {
    match HISTORY.lock() {
        Ok(history) => history.iter().cloned().collect(),
        Err(e) => {
            warn!("Failed to acquire lock for getting stats samples: {:?}", e);
            Vec::new() // Return empty vector instead of panicking
        }
    }
}
