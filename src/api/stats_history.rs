use std::collections::VecDeque;
use std::sync::Mutex;

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
    let mut history = HISTORY.lock().unwrap();
    history.push_back(sample);
    while history.len() > MAX_SAMPLES {
        history.pop_front();
    }
}

pub fn get_samples() -> Vec<StatsSample> {
    HISTORY.lock().unwrap().iter().cloned().collect()
}
