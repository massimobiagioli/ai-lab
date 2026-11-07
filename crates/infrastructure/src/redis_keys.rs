//! Redis key schema and TTLs (WP-04 §3.7).
//!
//! Redis is cache and ephemeral coordination only; it is never authoritative.
//! Every key is namespaced by environment and carries a TTL.

/// Retrieval cache TTL: 5 minutes.
pub const RETRIEVAL_CACHE_TTL_SECS: u64 = 300;
/// Embedding cache TTL: 24 hours.
pub const EMBEDDING_CACHE_TTL_SECS: u64 = 86_400;
/// Working-memory TTL: 2 hours.
pub const WORKING_MEMORY_TTL_SECS: u64 = 7_200;
/// Agent presence TTL: 60 seconds.
pub const PRESENCE_TTL_SECS: u64 = 60;
/// Fast budget-check TTL: 48 hours.
pub const BUDGET_TTL_SECS: u64 = 172_800;
/// Working memory keeps the last 40 messages per run.
pub const WORKING_MEMORY_MAX_MESSAGES: usize = 40;

fn key(env: &str, rest: &str) -> String {
    format!("ailab:{env}:{rest}")
}

/// `ailab:{env}:cache:retrieval:{hash}`
#[must_use]
pub fn retrieval_cache_key(env: &str, hash: &str) -> String {
    key(env, &format!("cache:retrieval:{hash}"))
}

/// `ailab:{env}:cache:embedding:{hash}`
#[must_use]
pub fn embedding_cache_key(env: &str, hash: &str) -> String {
    key(env, &format!("cache:embedding:{hash}"))
}

/// `ailab:{env}:work:{run_id}:messages`
#[must_use]
pub fn working_memory_key(env: &str, run_id: &str) -> String {
    key(env, &format!("work:{run_id}:messages"))
}

/// `ailab:{env}:rl:{scope}:{id}:{window}`
#[must_use]
pub fn rate_limit_key(env: &str, scope: &str, id: &str, window: &str) -> String {
    key(env, &format!("rl:{scope}:{id}:{window}"))
}

/// `ailab:{env}:lock:{name}`
#[must_use]
pub fn lock_key(env: &str, name: &str) -> String {
    key(env, &format!("lock:{name}"))
}

/// `ailab:{env}:presence:agent:{agent_id}`
#[must_use]
pub fn presence_key(env: &str, agent_id: &str) -> String {
    key(env, &format!("presence:agent:{agent_id}"))
}

/// `ailab:{env}:budget:{lab_id}:{day}`
#[must_use]
pub fn budget_key(env: &str, lab_id: &str, day: &str) -> String {
    key(env, &format!("budget:{lab_id}:{day}"))
}

/// `ailab:{env}:events`
#[must_use]
pub fn events_channel(env: &str) -> String {
    key(env, "events")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_namespaced_by_environment() {
        assert_eq!(
            retrieval_cache_key("test", "abc"),
            "ailab:test:cache:retrieval:abc"
        );
        assert_eq!(
            rate_limit_key("prod", "provider", "p1", "60"),
            "ailab:prod:rl:provider:p1:60"
        );
        assert_eq!(lock_key("dev", "scheduler"), "ailab:dev:lock:scheduler");
        assert_eq!(events_channel("dev"), "ailab:dev:events");
    }

    #[test]
    fn ttl_constants_match_the_contract() {
        assert_eq!(RETRIEVAL_CACHE_TTL_SECS, 300);
        assert_eq!(EMBEDDING_CACHE_TTL_SECS, 86_400);
        assert_eq!(WORKING_MEMORY_TTL_SECS, 7_200);
        assert_eq!(PRESENCE_TTL_SECS, 60);
        assert_eq!(BUDGET_TTL_SECS, 172_800);
        assert_eq!(WORKING_MEMORY_MAX_MESSAGES, 40);
    }
}
