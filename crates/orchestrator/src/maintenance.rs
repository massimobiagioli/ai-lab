//! Maintenance scheduler: due-job selection, bounded local handlers and the
//! `maintenance_jobs` bookkeeping (WP-23 T01/T05).

use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaintenanceJob {
    pub id: Uuid,
    pub name: String,
    pub cron: String,
    pub config: Value,
    pub next_run_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaintenanceStatus {
    Ok,
    Skipped,
    Failed,
}

impl MaintenanceStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MaintenanceOutcome {
    Ok(Option<String>),
    Skipped(String),
    Failed(String),
}

#[async_trait]
pub trait MaintenanceStore: Send + Sync {
    async fn due(&self, now: DateTime<Utc>) -> Result<Vec<MaintenanceJob>, String>;
    async fn record(
        &self,
        job: &MaintenanceJob,
        status: MaintenanceStatus,
        detail: Option<&str>,
        next_run_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<(), String>;
}

#[async_trait]
pub trait MaintenanceTasks: Send + Sync {
    async fn run(&self, job: &MaintenanceJob, now: DateTime<Utc>) -> MaintenanceOutcome;
}

#[async_trait]
pub trait StaleCardQuery: Send + Sync {
    async fn count_stale(&self, stale_days: u32, now: DateTime<Utc>) -> Result<u64, String>;
}

pub struct StaleCardScan<Q> {
    query: Q,
}

impl<Q> StaleCardScan<Q> {
    pub const fn new(query: Q) -> Self {
        Self { query }
    }
}

#[async_trait]
impl<Q: StaleCardQuery> MaintenanceTasks for StaleCardScan<Q> {
    async fn run(&self, job: &MaintenanceJob, now: DateTime<Utc>) -> MaintenanceOutcome {
        if job.name != "stale_card_scan" {
            return MaintenanceOutcome::Skipped(format!("handler not wired: {}", job.name));
        }
        let stale_days = job
            .config
            .get("stale_days")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(7);
        match self.query.count_stale(stale_days, now).await {
            Ok(count) => MaintenanceOutcome::Ok(Some(format!("stale_cards={count}"))),
            Err(error) => MaintenanceOutcome::Failed(error),
        }
    }
}

pub struct MaintenanceScheduler<S, T> {
    store: S,
    tasks: T,
}

impl<S, T> MaintenanceScheduler<S, T>
where
    S: MaintenanceStore,
    T: MaintenanceTasks,
{
    pub const fn new(store: S, tasks: T) -> Self {
        Self { store, tasks }
    }

    pub async fn tick(&self, now: DateTime<Utc>) -> Result<usize, String> {
        let jobs = self.store.due(now).await?;
        for job in &jobs {
            let outcome = self.tasks.run(job, now).await;
            let (status, detail) = match outcome {
                MaintenanceOutcome::Ok(detail) => (MaintenanceStatus::Ok, detail),
                MaintenanceOutcome::Skipped(detail) => (MaintenanceStatus::Skipped, Some(detail)),
                MaintenanceOutcome::Failed(detail) => (MaintenanceStatus::Failed, Some(detail)),
            };
            let next_run_at = next_run(&job.cron, now)?;
            self.store
                .record(job, status, detail.as_deref(), next_run_at, now)
                .await?;
        }
        Ok(jobs.len())
    }
}

pub fn next_run(cron: &str, now: DateTime<Utc>) -> Result<DateTime<Utc>, String> {
    let trimmed = cron.trim();
    let normalized = if trimmed.split_whitespace().count() == 5 {
        format!("0 {trimmed}")
    } else {
        trimmed.to_owned()
    };
    let schedule = cron::Schedule::from_str(&normalized).map_err(|error| error.to_string())?;
    schedule
        .after(&now)
        .next()
        .ok_or_else(|| "cron schedule has no next run".to_owned())
}

#[derive(Clone)]
pub struct PostgresMaintenance {
    pool: PgPool,
}

impl PostgresMaintenance {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MaintenanceStore for PostgresMaintenance {
    async fn due(&self, now: DateTime<Utc>) -> Result<Vec<MaintenanceJob>, String> {
        let rows = sqlx::query_as::<_, (Uuid, String, String, Value, Option<DateTime<Utc>>)>(
            "SELECT id, name, cron, config, next_run_at
               FROM maintenance_jobs
              WHERE enabled AND (next_run_at IS NULL OR next_run_at <= $1)
              ORDER BY name",
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| error.to_string())?;
        Ok(rows
            .into_iter()
            .map(|(id, name, cron, config, next_run_at)| MaintenanceJob {
                id,
                name,
                cron,
                config,
                next_run_at,
            })
            .collect())
    }

    async fn record(
        &self,
        job: &MaintenanceJob,
        status: MaintenanceStatus,
        detail: Option<&str>,
        next_run_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<(), String> {
        sqlx::query(
            "UPDATE maintenance_jobs
                SET last_run_at = $2, next_run_at = $3, last_status = $4, last_error = $5
              WHERE id = $1",
        )
        .bind(job.id)
        .bind(now)
        .bind(next_run_at)
        .bind(status.as_str())
        .bind(detail)
        .execute(&self.pool)
        .await
        .map_err(|error| error.to_string())?;
        Ok(())
    }
}

#[async_trait]
impl StaleCardQuery for PostgresMaintenance {
    async fn count_stale(&self, stale_days: u32, now: DateTime<Utc>) -> Result<u64, String> {
        let row: (i64,) = sqlx::query_as(
            "SELECT count(*)::bigint
               FROM cards
              WHERE column_name NOT IN ('DONE','ICEBOX')
                AND updated_at < $1 - make_interval(days => $2)",
        )
        .bind(now)
        .bind(i32::try_from(stale_days).unwrap_or(i32::MAX))
        .fetch_one(&self.pool)
        .await
        .map_err(|error| error.to_string())?;
        Ok(u64::try_from(row.0).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    type Recorded = (Uuid, MaintenanceStatus, Option<String>, DateTime<Utc>);

    #[derive(Default)]
    struct FakeStore {
        due: Vec<MaintenanceJob>,
        records: Arc<Mutex<Vec<Recorded>>>,
    }

    #[async_trait]
    impl MaintenanceStore for FakeStore {
        async fn due(&self, _now: DateTime<Utc>) -> Result<Vec<MaintenanceJob>, String> {
            Ok(self.due.clone())
        }

        async fn record(
            &self,
            job: &MaintenanceJob,
            status: MaintenanceStatus,
            detail: Option<&str>,
            next_run_at: DateTime<Utc>,
            _now: DateTime<Utc>,
        ) -> Result<(), String> {
            self.records.lock().unwrap().push((
                job.id,
                status,
                detail.map(str::to_owned),
                next_run_at,
            ));
            Ok(())
        }
    }

    struct FakeTasks {
        outcome: MaintenanceOutcome,
    }

    #[async_trait]
    impl MaintenanceTasks for FakeTasks {
        async fn run(&self, _job: &MaintenanceJob, _now: DateTime<Utc>) -> MaintenanceOutcome {
            match &self.outcome {
                MaintenanceOutcome::Ok(detail) => MaintenanceOutcome::Ok(detail.clone()),
                MaintenanceOutcome::Skipped(detail) => MaintenanceOutcome::Skipped(detail.clone()),
                MaintenanceOutcome::Failed(detail) => MaintenanceOutcome::Failed(detail.clone()),
            }
        }
    }

    fn job(name: &str) -> MaintenanceJob {
        MaintenanceJob {
            id: Uuid::new_v4(),
            name: name.to_owned(),
            cron: "0 7 * * *".to_owned(),
            config: serde_json::json!({}),
            next_run_at: None,
        }
    }

    #[test]
    fn seeded_schedules_have_a_next_run() {
        let now = Utc::now();
        for cron in [
            "0 3 * * *",
            "0 4 * * *",
            "0 6 * * 1",
            "0 7 * * *",
            "*/30 * * * *",
            "0 2 * * *",
            "* * * * *",
        ] {
            let next = next_run(cron, now).expect("next run");
            assert!(next > now, "{cron}");
        }
    }

    #[tokio::test]
    async fn tick_runs_due_jobs_and_records_the_next_run() {
        let store = FakeStore {
            due: vec![job("stale_card_scan")],
            records: Arc::new(Mutex::new(Vec::new())),
        };
        let records = store.records.clone();
        let scheduler = MaintenanceScheduler::new(
            store,
            FakeTasks {
                outcome: MaintenanceOutcome::Ok(Some("stale_cards=3".to_owned())),
            },
        );
        let now = Utc::now();
        assert_eq!(scheduler.tick(now).await.expect("tick"), 1);
        let recorded = records.lock().unwrap();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].1, MaintenanceStatus::Ok);
        assert_eq!(recorded[0].2.as_deref(), Some("stale_cards=3"));
        assert!(recorded[0].3 > now);
        drop(recorded);
    }

    #[tokio::test]
    async fn tick_records_a_failure_and_keeps_going() {
        let store = FakeStore {
            due: vec![job("stale_card_scan")],
            records: Arc::new(Mutex::new(Vec::new())),
        };
        let records = store.records.clone();
        let scheduler = MaintenanceScheduler::new(
            store,
            FakeTasks {
                outcome: MaintenanceOutcome::Failed("database down".to_owned()),
            },
        );
        assert_eq!(scheduler.tick(Utc::now()).await.expect("tick"), 1);
        let recorded = records.lock().unwrap();
        assert_eq!(recorded[0].1, MaintenanceStatus::Failed);
    }

    struct FakeQuery {
        count: Result<u64, String>,
    }

    #[async_trait]
    impl StaleCardQuery for FakeQuery {
        async fn count_stale(&self, _stale_days: u32, _now: DateTime<Utc>) -> Result<u64, String> {
            self.count.clone()
        }
    }

    #[tokio::test]
    async fn stale_scan_reports_the_count_and_skips_other_jobs() {
        let scan = StaleCardScan::new(FakeQuery { count: Ok(4) });
        let outcome = scan.run(&job("stale_card_scan"), Utc::now()).await;
        assert_eq!(
            outcome,
            MaintenanceOutcome::Ok(Some("stale_cards=4".to_owned()))
        );
        let outcome = scan.run(&job("memory_compaction"), Utc::now()).await;
        assert!(matches!(outcome, MaintenanceOutcome::Skipped(_)));
    }
}
