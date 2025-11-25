use std::collections::{HashSet, VecDeque};

use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use serde::{Deserialize, Serialize};
use snafu::GenerateImplicitData;
use std::path::PathBuf;
use std::sync::LazyLock;
use tracing::info;

use crate::{
    config::{BanCookie, CONFIG_PATH},
    error::ClewdrError,
};

#[derive(Debug, Serialize, Deserialize)]
struct QueueSnapshot {
    pending: Vec<BanCookie>,
    banned: Vec<BanCookie>,
    total_requests: u64,
}

static QUEUE_STATE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    CONFIG_PATH
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("queue_state.json")
});

#[derive(Debug, Serialize, Clone)]
pub struct BanQueueInfo {
    pub pending: Vec<BanCookie>,
    pub banned: Vec<BanCookie>,
    pub total_requests: u64,
}

#[derive(Debug)]
enum BanQueueMessage {
    Submit(BanCookie, RpcReplyPort<Result<(), ClewdrError>>),
    Pop(RpcReplyPort<Result<BanCookie, ClewdrError>>),
    MarkBanned(String, RpcReplyPort<Result<(), ClewdrError>>),
    Delete(String, RpcReplyPort<Result<(), ClewdrError>>),
    GetStatus(RpcReplyPort<BanQueueInfo>),
    ResetStats(RpcReplyPort<Result<(), ClewdrError>>),
    ClearPending(RpcReplyPort<Result<(), ClewdrError>>),
    ClearBanned(RpcReplyPort<Result<(), ClewdrError>>),
}

#[derive(Debug)]
struct BanQueueState {
    pending: VecDeque<BanCookie>,
    banned: HashSet<BanCookie>,
    total_requests: u64,
}

impl BanQueueState {
    fn new() -> Self {
        Self {
            pending: VecDeque::new(),
            banned: HashSet::new(),
            total_requests: 0,
        }
    }

    fn from_snapshot(snapshot: QueueSnapshot) -> Self {
        let mut pending = VecDeque::new();
        for c in snapshot.pending {
            pending.push_back(c);
        }
        let banned = snapshot.banned.into_iter().collect();
        Self {
            pending,
            banned,
            total_requests: snapshot.total_requests,
        }
    }

    fn snapshot(&self) -> QueueSnapshot {
        QueueSnapshot {
            pending: self.pending.iter().cloned().collect(),
            banned: self.banned.iter().cloned().collect(),
            total_requests: self.total_requests,
        }
    }

    async fn persist(&self) {
        let snapshot = self.snapshot();
        match serde_json::to_vec_pretty(&snapshot) {
            Ok(buf) => {
                if let Err(e) = tokio::fs::write(&*QUEUE_STATE_PATH, buf).await {
                    tracing::warn!("Failed to persist queue state: {}", e);
                }
            }
            Err(e) => tracing::warn!("Failed to serialize queue state: {}", e),
        }
    }

    async fn load_snapshot() -> Option<QueueSnapshot> {
        if !QUEUE_STATE_PATH.exists() {
            return None;
        }
        let bytes = tokio::fs::read(&*QUEUE_STATE_PATH).await.ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    fn log(&self) {
        info!(
            "Queue status: {} pending, {} banned, {} total requests",
            self.pending.len(),
            self.banned.len(),
            self.total_requests
        );
    }
}

struct BanQueueActor;

impl Actor for BanQueueActor {
    type Msg = BanQueueMessage;
    type State = BanQueueState;
    type Arguments = Option<BanQueueState>;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        arguments: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let state = arguments.unwrap_or_else(BanQueueState::new);
        info!("BanQueue actor started");
        Ok(state)
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            BanQueueMessage::Submit(cookie, reply_port) => {
                // Check if cookie already exists
                if state.pending.iter().any(|c| c == &cookie) || state.banned.contains(&cookie) {
                    reply_port.send(Err(ClewdrError::BadRequest {
                        msg: "Cookie already exists",
                    }))?;
                    return Ok(());
                }

                state.pending.push_back(cookie.clone());
                info!("Cookie submitted: {}", cookie.cookie.ellipse());
                state.log();
                state.persist().await;
                reply_port.send(Ok(()))?;
            }
            BanQueueMessage::Pop(reply_port) => {
                match state.pending.pop_front() {
                    Some(mut cookie) => {
                        cookie.mark_used();
                        state.total_requests += 1;
                        // Put it back at the end for reuse
                        state.pending.push_back(cookie.clone());
                        reply_port.send(Ok(cookie))?;
                        state.persist().await;
                    }
                    None => {
                        reply_port.send(Err(ClewdrError::NoCookieAvailable))?;
                    }
                }
            }
            BanQueueMessage::MarkBanned(cookie_str, reply_port) => {
                // Remove from pending and add to banned
                if let Some(pos) = state
                    .pending
                    .iter()
                    .position(|c| c.cookie.to_string() == cookie_str)
                {
                    let mut cookie = state
                        .pending
                        .remove(pos)
                        .expect("Cookie at found position should exist");
                    cookie.mark_banned();
                    info!("Cookie banned: {}", cookie.cookie.ellipse());
                    state.banned.insert(cookie);
                    state.log();
                    state.persist().await;
                    reply_port.send(Ok(()))?;
                } else {
                    reply_port.send(Err(ClewdrError::UnexpectedNone {
                        msg: "Cookie not found in queue",
                    }))?;
                }
            }
            BanQueueMessage::Delete(cookie_str, reply_port) => {
                // Remove from pending
                let pending_removed = state
                    .pending
                    .iter()
                    .position(|c| c.cookie.to_string() == cookie_str)
                    .map(|pos| state.pending.remove(pos));

                // Remove from banned
                let banned_removed = state
                    .banned
                    .iter()
                    .find(|c| c.cookie.to_string() == cookie_str)
                    .cloned()
                    .and_then(|c| {
                        if state.banned.remove(&c) {
                            Some(c)
                        } else {
                            None
                        }
                    });

                if pending_removed.is_some() || banned_removed.is_some() {
                    info!("Cookie deleted: {}", cookie_str);
                    state.log();
                    state.persist().await;
                    reply_port.send(Ok(()))?;
                } else {
                    reply_port.send(Err(ClewdrError::UnexpectedNone {
                        msg: "Cookie not found",
                    }))?;
                }
            }
            BanQueueMessage::GetStatus(reply_port) => {
                let info = BanQueueInfo {
                    pending: state.pending.iter().cloned().collect(),
                    banned: state.banned.iter().cloned().collect(),
                    total_requests: state.total_requests,
                };
                reply_port.send(info)?;
            }
            BanQueueMessage::ResetStats(reply_port) => {
                state.total_requests = 0;
                for c in state.pending.iter_mut() {
                    c.requests_sent = 0;
                    c.last_used_at = None;
                }
                let mut rebuilt = HashSet::new();
                for mut c in state.banned.drain() {
                    c.requests_sent = 0;
                    c.last_used_at = None;
                    rebuilt.insert(c);
                }
                state.banned = rebuilt;
                state.persist().await;
                reply_port.send(Ok(()))?;
            }
            BanQueueMessage::ClearPending(reply_port) => {
                state.pending.clear();
                state.persist().await;
                reply_port.send(Ok(()))?;
            }
            BanQueueMessage::ClearBanned(reply_port) => {
                state.banned.clear();
                state.persist().await;
                reply_port.send(Ok(()))?;
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct BanQueueHandle {
    actor_ref: ActorRef<BanQueueMessage>,
}

impl BanQueueHandle {
    pub async fn start() -> Result<Self, ractor::SpawnErr> {
        let initial_state = BanQueueState::load_snapshot()
            .await
            .map(BanQueueState::from_snapshot);
        let (actor_ref, _join_handle) = Actor::spawn(None, BanQueueActor, initial_state).await?;
        Ok(Self { actor_ref })
    }

    pub async fn submit(&self, cookie: BanCookie) -> Result<(), ClewdrError> {
        ractor::call!(self.actor_ref, BanQueueMessage::Submit, cookie).map_err(|e| {
            ClewdrError::RactorError {
                loc: snafu::Location::generate(),
                msg: format!("Failed to submit cookie: {e}"),
            }
        })?
    }

    pub async fn pop(&self) -> Result<BanCookie, ClewdrError> {
        ractor::call!(self.actor_ref, BanQueueMessage::Pop).map_err(|e| {
            ClewdrError::RactorError {
                loc: snafu::Location::generate(),
                msg: format!("Failed to pop cookie: {e}"),
            }
        })?
    }

    pub async fn mark_banned(&self, cookie: String) -> Result<(), ClewdrError> {
        ractor::call!(self.actor_ref, BanQueueMessage::MarkBanned, cookie).map_err(|e| {
            ClewdrError::RactorError {
                loc: snafu::Location::generate(),
                msg: format!("Failed to mark banned: {e}"),
            }
        })?
    }

    pub async fn delete(&self, cookie: String) -> Result<(), ClewdrError> {
        ractor::call!(self.actor_ref, BanQueueMessage::Delete, cookie).map_err(|e| {
            ClewdrError::RactorError {
                loc: snafu::Location::generate(),
                msg: format!("Failed to delete cookie: {e}"),
            }
        })?
    }

    pub async fn get_status(&self) -> Result<BanQueueInfo, ClewdrError> {
        ractor::call!(self.actor_ref, BanQueueMessage::GetStatus).map_err(|e| {
            ClewdrError::RactorError {
                loc: snafu::Location::generate(),
                msg: format!("Failed to get status: {e}"),
            }
        })
    }

    pub async fn reset_stats(&self) -> Result<(), ClewdrError> {
        ractor::call!(self.actor_ref, BanQueueMessage::ResetStats).map_err(|e| {
            ClewdrError::RactorError {
                loc: snafu::Location::generate(),
                msg: format!("Failed to reset stats: {e}"),
            }
        })?
    }

    pub async fn clear_pending(&self) -> Result<(), ClewdrError> {
        ractor::call!(self.actor_ref, BanQueueMessage::ClearPending).map_err(|e| {
            ClewdrError::RactorError {
                loc: snafu::Location::generate(),
                msg: format!("Failed to clear pending queue: {e}"),
            }
        })?
    }

    pub async fn clear_banned(&self) -> Result<(), ClewdrError> {
        ractor::call!(self.actor_ref, BanQueueMessage::ClearBanned).map_err(|e| {
            ClewdrError::RactorError {
                loc: snafu::Location::generate(),
                msg: format!("Failed to clear banned queue: {e}"),
            }
        })?
    }
}
