//! Scheduler eligibility, leases, retries, budgets and crash recovery.

use std::collections::BTreeMap;
use std::sync::Mutex;

use ailab_domain::{AgentId, CardId, Money, RunId};
use chrono::{DateTime, Duration, Utc};
use thiserror::Error;

/// Lease failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LeaseError {
    /// Another worker holds the lease.
    #[error("card is already leased")]
    Conflict,
    /// No active lease.
    #[error("lease not found")]
    NotFound,
}

/// A per-card lease.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lease {
    /// Run.
    pub run_id: RunId,
    /// Card.
    pub card_id: CardId,
    /// Agent.
    pub agent_id: AgentId,
    /// Expiry.
    pub expires_at: DateTime<Utc>,
    /// Last heartbeat.
    pub heartbeat_at: DateTime<Utc>,
}

/// In-memory lease manager with a 900 s TTL.
#[derive(Debug)]
pub struct LeaseManager {
    leases: Mutex<BTreeMap<CardId, Lease>>,
    ttl_s: i64,
}

impl Default for LeaseManager {
    fn default() -> Self {
        Self::new(900)
    }
}

impl LeaseManager {
    /// Create a manager with a TTL.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(ttl_s: i64) -> Self {
        Self {
            leases: Mutex::new(BTreeMap::new()),
            ttl_s,
        }
    }

    /// Acquire a lease; exactly one worker wins.
    pub fn acquire(
        &self,
        card_id: CardId,
        run_id: RunId,
        agent_id: AgentId,
        now: DateTime<Utc>,
    ) -> Result<Lease, LeaseError> {
        let mut leases = self.leases.lock().map_err(|_| LeaseError::Conflict)?;
        if let Some(existing) = leases.get(&card_id)
            && existing.expires_at > now
        {
            return Err(LeaseError::Conflict);
        }
        let lease = Lease {
            run_id,
            card_id,
            agent_id,
            expires_at: now + Duration::seconds(self.ttl_s),
            heartbeat_at: now,
        };
        leases.insert(card_id, lease.clone());
        drop(leases);
        Ok(lease)
    }

    /// Renew a lease and heartbeat.
    pub fn renew(&self, card_id: CardId, now: DateTime<Utc>) -> Result<(), LeaseError> {
        let mut leases = self.leases.lock().map_err(|_| LeaseError::NotFound)?;
        let lease = leases.get_mut(&card_id).ok_or(LeaseError::NotFound)?;
        if lease.expires_at <= now {
            return Err(LeaseError::NotFound);
        }
        lease.heartbeat_at = now;
        lease.expires_at = now + Duration::seconds(self.ttl_s);
        drop(leases);
        Ok(())
    }

    /// Release a lease.
    pub fn release(&self, card_id: CardId) {
        if let Ok(mut leases) = self.leases.lock() {
            leases.remove(&card_id);
        }
    }

    /// Read an active lease.
    #[must_use]
    pub fn get(&self, card_id: CardId, now: DateTime<Utc>) -> Option<Lease> {
        self.leases.lock().ok().and_then(|leases| {
            leases
                .get(&card_id)
                .filter(|lease| lease.expires_at > now)
                .cloned()
        })
    }

    /// Remove and return expired leases.
    pub fn reap(&self, now: DateTime<Utc>) -> Vec<CardId> {
        let mut expired = Vec::new();
        if let Ok(mut leases) = self.leases.lock() {
            let keys: Vec<CardId> = leases
                .iter()
                .filter(|(_, lease)| lease.expires_at <= now)
                .map(|(card_id, _)| *card_id)
                .collect();
            for card_id in keys {
                leases.remove(&card_id);
                expired.push(card_id);
            }
        }
        expired
    }
}

/// Eligibility result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Eligibility {
    /// True when every condition holds.
    pub eligible: bool,
    /// Human-readable reasons.
    pub reasons: Vec<String>,
}

/// Evaluate scheduler eligibility.
#[must_use]
#[allow(clippy::fn_params_excessive_bools)]
pub fn evaluate_eligibility(
    card_ready: bool,
    agent_active: bool,
    budget_available: bool,
    wip_ok: bool,
    already_leased: bool,
) -> Eligibility {
    let mut reasons = Vec::new();
    if !card_ready {
        reasons.push("card is not READY".to_owned());
    }
    if !agent_active {
        reasons.push("agent is not active".to_owned());
    }
    if !budget_available {
        reasons.push("budget exhausted".to_owned());
    }
    if !wip_ok {
        reasons.push("WIP limit reached".to_owned());
    }
    if already_leased {
        reasons.push("card is already leased".to_owned());
    }
    Eligibility {
        eligible: reasons.is_empty(),
        reasons,
    }
}

/// Run state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    /// Queued.
    Queued,
    /// Running.
    Running,
    /// Succeeded.
    Succeeded,
    /// Failed.
    Failed,
    /// Cancelled.
    Cancelled,
    /// Timed out.
    TimedOut,
}

/// Run failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RunError {
    /// Invalid state transition.
    #[error("invalid run transition from {0:?}")]
    InvalidState(RunState),
}

/// One attempt by one agent on one card.
#[derive(Debug, Clone)]
pub struct Run {
    /// Run id.
    pub id: RunId,
    /// Card.
    pub card_id: CardId,
    /// Agent.
    pub agent_id: AgentId,
    /// Attempt number.
    pub attempt: u8,
    /// State.
    pub state: RunState,
    /// Started at.
    pub started_at: Option<DateTime<Utc>>,
    /// Ended at.
    pub ended_at: Option<DateTime<Utc>>,
    /// Last heartbeat.
    pub heartbeat_at: Option<DateTime<Utc>>,
}

#[allow(clippy::missing_const_for_fn)]
impl Run {
    /// New queued run at attempt 1.
    #[must_use]
    pub fn new(id: RunId, card_id: CardId, agent_id: AgentId) -> Self {
        Self {
            id,
            card_id,
            agent_id,
            attempt: 1,
            state: RunState::Queued,
            started_at: None,
            ended_at: None,
            heartbeat_at: None,
        }
    }

    /// Start the run.
    pub fn start(&mut self, now: DateTime<Utc>) -> Result<(), RunError> {
        if self.state != RunState::Queued {
            return Err(RunError::InvalidState(self.state));
        }
        self.state = RunState::Running;
        self.started_at = Some(now);
        self.heartbeat_at = Some(now);
        Ok(())
    }

    /// Heartbeat the run.
    pub fn heartbeat(&mut self, now: DateTime<Utc>) {
        self.heartbeat_at = Some(now);
    }

    /// Mark succeeded.
    pub fn succeed(&mut self, now: DateTime<Utc>) -> Result<(), RunError> {
        self.finish(RunState::Succeeded, now)
    }

    /// Mark failed.
    pub fn fail(&mut self, now: DateTime<Utc>) -> Result<(), RunError> {
        self.finish(RunState::Failed, now)
    }

    /// Mark timed out.
    pub fn timeout(&mut self, now: DateTime<Utc>) -> Result<(), RunError> {
        self.finish(RunState::TimedOut, now)
    }

    /// Cancel a queued or running run.
    pub fn cancel(&mut self, now: DateTime<Utc>) -> Result<(), RunError> {
        if !matches!(self.state, RunState::Queued | RunState::Running) {
            return Err(RunError::InvalidState(self.state));
        }
        self.state = RunState::Cancelled;
        self.ended_at = Some(now);
        Ok(())
    }

    fn finish(&mut self, state: RunState, now: DateTime<Utc>) -> Result<(), RunError> {
        if self.state != RunState::Running {
            return Err(RunError::InvalidState(self.state));
        }
        self.state = state;
        self.ended_at = Some(now);
        Ok(())
    }
}

/// Error class used by retry decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Transient; retry.
    Transient,
    /// Permanent; dead-letter.
    Permanent,
    /// Policy; dead-letter.
    Policy,
}

/// Retry decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryDecision {
    /// Requeue with a backoff.
    Requeue {
        /// Next attempt.
        attempt: u8,
        /// Backoff in milliseconds.
        backoff_ms: u64,
    },
    /// Dead-letter the card to ICEBOX.
    DeadLetter,
}

/// Decide whether to requeue or dead-letter.
#[must_use]
pub fn retry_decision(attempt: u8, error: ErrorKind) -> RetryDecision {
    if error != ErrorKind::Transient || attempt >= 3 {
        return RetryDecision::DeadLetter;
    }
    let shift = u32::from(attempt.saturating_sub(1));
    let backoff_ms = 100_u64.saturating_mul(1_u64 << shift.min(10)).min(5_000);
    RetryDecision::Requeue {
        attempt: attempt + 1,
        backoff_ms,
    }
}

/// Per-run budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunBudget {
    /// Limit.
    pub limit: Money,
    /// Spent.
    pub spent: Money,
}

impl RunBudget {
    /// Create a budget.
    #[must_use]
    pub const fn new(limit: Money) -> Self {
        Self {
            limit,
            spent: Money::ZERO,
        }
    }

    /// True when the run may start.
    #[must_use]
    pub const fn can_start(self) -> bool {
        self.spent.microusd() < self.limit.microusd()
    }

    /// Charge a cost; reject when it would exceed the limit.
    pub fn charge(&mut self, cost: Money) -> Result<(), String> {
        let next = self
            .spent
            .checked_add(cost)
            .map_err(|error| error.to_string())?;
        if next > self.limit {
            return Err("budget exceeded".to_owned());
        }
        self.spent = next;
        Ok(())
    }
}

/// Reconcile running runs whose heartbeat is older than the TTL.
#[must_use]
pub fn reconcile(runs: &mut [Run], now: DateTime<Utc>, ttl_s: i64) -> Vec<RunId> {
    let mut timed_out = Vec::new();
    for run in runs {
        if run.state != RunState::Running {
            continue;
        }
        let stale = run
            .heartbeat_at
            .is_none_or(|heartbeat| (now - heartbeat).num_seconds() > ttl_s);
        if stale {
            run.state = RunState::TimedOut;
            run.ended_at = Some(now);
            timed_out.push(run.id);
        }
    }
    timed_out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
            .single()
            .expect("timestamp")
    }

    #[test]
    fn eligibility_reports_every_reason() {
        let result = evaluate_eligibility(false, false, false, false, true);
        assert!(!result.eligible);
        assert_eq!(result.reasons.len(), 5);
        assert!(evaluate_eligibility(true, true, true, true, false).eligible);
    }

    #[test]
    fn one_worker_wins_the_lease() {
        let manager = LeaseManager::default();
        let card = CardId::generate();
        let first = manager
            .acquire(card, RunId::generate(), AgentId::generate(), now())
            .expect("first");
        assert_eq!(first.card_id, card);
        assert_eq!(
            manager.acquire(card, RunId::generate(), AgentId::generate(), now()),
            Err(LeaseError::Conflict)
        );
        manager.release(card);
        assert!(
            manager
                .acquire(card, RunId::generate(), AgentId::generate(), now())
                .is_ok()
        );
    }

    #[test]
    fn lease_expiry_and_reap() {
        let manager = LeaseManager::new(10);
        let card = CardId::generate();
        manager
            .acquire(card, RunId::generate(), AgentId::generate(), now())
            .expect("lease");
        assert!(manager.get(card, now() + Duration::seconds(11)).is_none());
        assert_eq!(manager.reap(now() + Duration::seconds(11)), vec![card]);
    }

    #[test]
    fn run_state_machine() {
        let mut run = Run::new(RunId::generate(), CardId::generate(), AgentId::generate());
        assert!(run.start(now()).is_ok());
        assert!(run.succeed(now()).is_ok());
        assert_eq!(run.state, RunState::Succeeded);
        assert!(run.start(now()).is_err());
    }

    #[test]
    fn retries_are_bounded_and_classified() {
        assert_eq!(
            retry_decision(1, ErrorKind::Transient),
            RetryDecision::Requeue {
                attempt: 2,
                backoff_ms: 100
            }
        );
        assert_eq!(
            retry_decision(3, ErrorKind::Transient),
            RetryDecision::DeadLetter
        );
        assert_eq!(
            retry_decision(1, ErrorKind::Permanent),
            RetryDecision::DeadLetter
        );
    }

    #[test]
    fn budget_stops_before_exhaustion() {
        let mut budget = RunBudget::new(Money::from_microusd(1_000).expect("limit"));
        assert!(budget.can_start());
        budget
            .charge(Money::from_microusd(900).expect("cost"))
            .expect("charge");
        assert!(
            budget
                .charge(Money::from_microusd(200).expect("cost"))
                .is_err()
        );
    }

    #[test]
    fn reconcile_times_out_stale_runs() {
        let mut run = Run::new(RunId::generate(), CardId::generate(), AgentId::generate());
        run.start(now()).expect("start");
        let id = run.id;
        let timed_out = reconcile(
            std::slice::from_mut(&mut run),
            now() + Duration::seconds(901),
            900,
        );
        assert_eq!(timed_out, vec![id]);
        assert_eq!(run.state, RunState::TimedOut);
    }
}
