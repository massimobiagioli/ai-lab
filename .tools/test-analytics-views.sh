#!/usr/bin/env bash
# Verify migrations/analytics_views.sql against fixed fixtures on a disposable
# PostgreSQL + pgvector container.
#
# Usage: .tools/test-analytics-views.sh [--keep]
#   --keep   leave the container running for inspection (prints its name)
#
# Requirements: docker (running), bash. No host PostgreSQL client is needed.
# Exit codes: 0 all assertions pass, 1 an assertion failed, 2 environment issue.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DDL="$ROOT/migrations/0001_init.sql"
VIEWS="$ROOT/migrations/0002_analytics_views.sql"

KEEP=0
[ "${1:-}" = "--keep" ] && KEEP=1

for f in "$DDL" "$VIEWS"; do
  if [ ! -f "$f" ]; then
    echo "ENV ERROR: missing $f" >&2
    exit 2
  fi
done

if ! command -v docker >/dev/null 2>&1 || ! docker info >/dev/null 2>&1; then
  echo "ENV ERROR: docker is not available or not running" >&2
  exit 2
fi

IMAGE="${AILAB_TEST_PG_IMAGE:-pgvector/pgvector:pg18}"
if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  IMAGE="pgvector/pgvector:pg17"
fi

NAME="ailab-analytics-test-$$"
cleanup() {
  if [ "$KEEP" -eq 1 ]; then
    echo "container kept: $NAME"
  else
    docker rm -f "$NAME" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

docker run -d --name "$NAME" \
  -e POSTGRES_PASSWORD=test -e POSTGRES_DB=ailab \
  "$IMAGE" >/dev/null || { echo "ENV ERROR: cannot start container" >&2; exit 2; }

ready=0
for _ in $(seq 1 60); do
  if docker exec "$NAME" psql -U postgres -d ailab -tAc 'select 1' >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 1
done
if [ "$ready" -ne 1 ]; then
  echo "ENV ERROR: PostgreSQL did not become ready" >&2
  docker logs "$NAME" >&2 || true
  exit 2
fi
sleep 2

psql_run() {
  docker exec -i "$NAME" psql -U postgres -d ailab -v ON_ERROR_STOP=1 -q "$@"
}

if ! psql_run < "$DDL"; then
  echo "ENV ERROR: schema DDL failed" >&2
  exit 2
fi
if ! psql_run < "$VIEWS"; then
  echo "FAIL: analytics_views.sql failed to apply" >&2
  exit 1
fi
if ! psql_run < "$ROOT/.tools/fixtures/analytics-seed.sql"; then
  echo "ENV ERROR: analytics fixture failed" >&2
  exit 2
fi

PASS=0
FAIL=0
assert_eq() {
  local label="$1" expected="$2" actual="$3"
  if [ "$expected" = "$actual" ]; then
    printf 'PASS  %s = %s\n' "$label" "$actual"
    PASS=$((PASS + 1))
  else
    printf 'FAIL  %s expected=%s actual=%s\n' "$label" "$expected" "$actual"
    FAIL=$((FAIL + 1))
  fi
}

q() {
  psql_run -tA -c "SET TIME ZONE 'UTC'; $1" | tr -d '\r'
}

assert_eq "lead_time C1 seconds" "3600.000000" "$(q "SELECT lead_time_seconds FROM analytics.card_lead_time WHERE card_id='44444444-4444-7444-8444-444444444441'")"
assert_eq "lead_time C2 seconds" "1800.000000" "$(q "SELECT lead_time_seconds FROM analytics.card_lead_time WHERE card_id='44444444-4444-7444-8444-444444444442'")"
assert_eq "lead_time C3 seconds" "2400.000000" "$(q "SELECT lead_time_seconds FROM analytics.card_lead_time WHERE card_id='44444444-4444-7444-8444-444444444443'")"
assert_eq "done cards" "3" "$(q "SELECT count(*) FROM analytics.card_lead_time")"

assert_eq "cycle_time C1 seconds" "3000.000000" "$(q "SELECT cycle_time_seconds FROM analytics.card_cycle_time WHERE card_id='44444444-4444-7444-8444-444444444441'")"
assert_eq "cycle_time C2 seconds" "1500.000000" "$(q "SELECT cycle_time_seconds FROM analytics.card_cycle_time WHERE card_id='44444444-4444-7444-8444-444444444442'")"

assert_eq "throughput week" "2025-12-29" "$(q "SELECT to_char(week_start, 'YYYY-MM-DD') FROM analytics.throughput_weekly")"
assert_eq "throughput cards" "3" "$(q "SELECT cards_done FROM analytics.throughput_weekly")"

assert_eq "wip DOING" "1" "$(q "SELECT card_count FROM analytics.wip_current WHERE column_name='DOING'")"
assert_eq "wip REVIEW" "1" "$(q "SELECT card_count FROM analytics.wip_current WHERE column_name='REVIEW'")"
assert_eq "wip rows" "2" "$(q "SELECT count(*) FROM analytics.wip_current")"

assert_eq "blocked C3 seconds" "600.000000" "$(q "SELECT blocked_seconds_closed FROM analytics.blocked_time WHERE card_id='44444444-4444-7444-8444-444444444443'")"
assert_eq "blocked open intervals" "0" "$(q "SELECT COALESCE(sum(open_blocked_intervals),0) FROM analytics.blocked_time")"

assert_eq "flow efficiency C1" "0.016667" "$(q "SELECT round(flow_efficiency::numeric, 6) FROM analytics.flow_efficiency WHERE card_id='44444444-4444-7444-8444-444444444441'")"
assert_eq "flow efficiency C2" "0.016667" "$(q "SELECT round(flow_efficiency::numeric, 6) FROM analytics.flow_efficiency WHERE card_id='44444444-4444-7444-8444-444444444442'")"

assert_eq "rework cards" "1" "$(q "SELECT rework_cards FROM analytics.rework_rate")"
assert_eq "rework rate" "0.333333" "$(q "SELECT round(rework_rate, 6) FROM analytics.rework_rate")"

assert_eq "cost total" "3000000" "$(q "SELECT cost_microusd FROM analytics.cost_per_card")"
assert_eq "cost average" "1000000" "$(q "SELECT round(avg_cost_microusd)::bigint FROM analytics.cost_per_card")"
assert_eq "cost median" "1000000" "$(q "SELECT round(median_cost_microusd)::bigint FROM analytics.cost_per_card")"

assert_eq "autonomous done" "2" "$(q "SELECT autonomous_done_cards FROM analytics.autonomous_success")"
assert_eq "autonomous rate" "0.666667" "$(q "SELECT round(autonomous_success_rate, 6) FROM analytics.autonomous_success")"

assert_eq "lead p50" "2400.000000" "$(q "SELECT round(p50_lead_time_seconds::numeric, 6) FROM analytics.flow_summary")"
assert_eq "lead p95" "3480.000000" "$(q "SELECT round(p95_lead_time_seconds::numeric, 6) FROM analytics.flow_summary")"
assert_eq "cycle p50" "2100.000000" "$(q "SELECT round(p50_cycle_time_seconds::numeric, 6) FROM analytics.flow_summary")"

echo
if [ "$FAIL" -ne 0 ]; then
  echo "ANALYTICS VIEWS: FAILED ($PASS passed, $FAIL failed)"
  exit 1
fi
echo "ANALYTICS VIEWS: OK ($PASS assertions)"
exit 0
