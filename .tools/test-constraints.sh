#!/usr/bin/env bash
# Constraint and optimistic-concurrency tests against the real migrations.
#
# Negative cases for every constraint family (CHECK, UNIQUE, partial UNIQUE,
# FOREIGN KEY, vector dimension) across the core aggregates, plus a two-step
# UPDATE with `version` proving the optimistic-concurrency guard.
#
# Usage: .tools/test-constraints.sh
# Exit: 0 all cases behave as expected, 1 a case failed, 2 environment error.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INIT="$ROOT/migrations/0001_init.sql"
VIEWS="$ROOT/migrations/0002_analytics_views.sql"
IMAGE="${AILAB_TEST_PG_IMAGE:-pgvector/pgvector:pg18}"

for f in "$INIT" "$VIEWS"; do
  [ -f "$f" ] || { echo "ENV ERROR: missing $f" >&2; exit 2; }
done
command -v docker >/dev/null 2>&1 || { echo "ENV ERROR: docker missing" >&2; exit 2; }
docker image inspect "$IMAGE" >/dev/null 2>&1 || IMAGE="pgvector/pgvector:pg17"

NAME="ailab-constraints-test-$$"
cleanup() { docker rm -f "$NAME" >/dev/null 2>&1 || true; }
trap cleanup EXIT

docker run -d --name "$NAME" -e POSTGRES_PASSWORD=test -e POSTGRES_DB=ailab "$IMAGE" >/dev/null || exit 2
for _ in $(seq 1 60); do
  docker exec "$NAME" psql -U postgres -d ailab -tAc 'select 1' >/dev/null 2>&1 && break
  sleep 1
done
sleep 1
docker exec -i "$NAME" psql -U postgres -d ailab -v ON_ERROR_STOP=1 -q < "$INIT" >/dev/null
docker exec -i "$NAME" psql -U postgres -d ailab -v ON_ERROR_STOP=1 -q < "$VIEWS" >/dev/null

psql_c() { docker exec -i "$NAME" psql -U postgres -d ailab -v ON_ERROR_STOP=1 -q "$@"; }

psql_c >/dev/null <<'SQL'
INSERT INTO labs (id, workspace_id, name, slug)
VALUES ('aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaaa',
        '00000000-0000-7000-8000-000000000001', 'Test Lab', 'test-lab');
INSERT INTO agents (id, lab_id, name, role, character_id, description, autonomy_level)
VALUES
  ('a1000000-0000-7000-8000-000000000001', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaaa',
   'Dev Test', 'dev', 'scrappy_dev',
   'Fixture developer agent with a sufficiently long description used only to exercise database constraints and invariants in the data platform test suite.',
   2),
  ('a1000000-0000-7000-8000-000000000002', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaaa',
   'QA Test', 'qa', 'precise_consultant',
   'Fixture QA agent with a sufficiently long description used only to exercise database constraints and invariants in the data platform test suite.',
   2);
INSERT INTO projects (id, lab_id, name, slug, key_prefix, source_ref)
VALUES ('bbbbbbbb-bbbb-7bbb-8bbb-bbbbbbbbbbbb',
        'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaaa',
        'Test Project', 'test-project', 'TST', 'example/test-project');
INSERT INTO boards (id, project_id)
VALUES ('cccccccc-cccc-7ccc-8ccc-cccccccccccc',
        'bbbbbbbb-bbbb-7bbb-8bbb-bbbbbbbbbbbb');
INSERT INTO providers (id, workspace_id, name, kind, base_url, api_key_ciphertext, api_key_nonce, api_key_last4)
VALUES ('e0000000-0000-7000-8000-000000000001',
        '00000000-0000-7000-8000-000000000001', 'DeepSeek', 'deepseek',
        'https://api.deepseek.com', '\x00', '\x00', 'abcd');
INSERT INTO models (id, provider_id, name, display_name, context_window, max_output_tokens)
VALUES ('f0000000-0000-7000-8000-000000000001',
        'e0000000-0000-7000-8000-000000000001', 'deepseek-flash', 'DeepSeek Flash', 128000, 8192);
INSERT INTO memory_documents (id, lab_id, scope, kind, title, content, content_hash, created_by)
VALUES ('a2000000-0000-7000-8000-000000000001', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaaa',
        'lab', 'note', 'Lab note', 'Fixture lab memory document', 'hash-1', 'system');
INSERT INTO cards (id, board_id, project_id, card_key, seq, title, column_name, priority, estimate)
VALUES ('d0000000-0000-7000-8000-000000000100', 'cccccccc-cccc-7ccc-8ccc-cccccccccccc',
        'bbbbbbbb-bbbb-7bbb-8bbb-bbbbbbbbbbbb', 'TST-100', 100, 'Base card', 'TODO', 'should', 3);
SQL

FAIL=0
PASS=0
expect_fail() {
  local label="$1" sql="$2"
  if psql_c -c "$sql" >/dev/null 2>&1; then
    printf 'FAIL  %s (expected rejection)\n' "$label"
    FAIL=1
  else
    printf 'PASS  %s\n' "$label"
    PASS=$((PASS + 1))
  fi
}

expect_fail "duplicate board per project" \
  "INSERT INTO boards (id, project_id) VALUES ('cccccccc-cccc-7ccc-8ccc-cccccccccccd', 'bbbbbbbb-bbbb-7bbb-8bbb-bbbbbbbbbbbb');"

expect_fail "READY without assignee" \
  "INSERT INTO cards (id, board_id, project_id, card_key, seq, title, column_name, priority, estimate, due_date)
   VALUES ('d0000000-0000-7000-8000-000000000001', 'cccccccc-cccc-7ccc-8ccc-cccccccccccc',
           'bbbbbbbb-bbbb-7bbb-8bbb-bbbbbbbbbbbb', 'TST-1', 1, 'Bad READY', 'READY', 'must', 3, now() + interval '1 day');"

expect_fail "REVIEW with reviewer equal to assignee" \
  "INSERT INTO cards (id, board_id, project_id, card_key, seq, title, column_name, priority, estimate, assignee_id, reviewer_id, pr_url)
   VALUES ('d0000000-0000-7000-8000-000000000002', 'cccccccc-cccc-7ccc-8ccc-cccccccccccc',
           'bbbbbbbb-bbbb-7bbb-8bbb-bbbbbbbbbbbb', 'TST-2', 2, 'Bad REVIEW', 'REVIEW', 'must', 3,
           'a1000000-0000-7000-8000-000000000001', 'a1000000-0000-7000-8000-000000000001', 'https://example.test/pr/2');"

expect_fail "DONE without merge SHA" \
  "INSERT INTO cards (id, board_id, project_id, card_key, seq, title, column_name, priority, estimate, assignee_id, reviewer_id)
   VALUES ('d0000000-0000-7000-8000-000000000003', 'cccccccc-cccc-7ccc-8ccc-cccccccccccc',
           'bbbbbbbb-bbbb-7bbb-8bbb-bbbbbbbbbbbb', 'TST-3', 3, 'Bad DONE', 'DONE', 'must', 3,
           'a1000000-0000-7000-8000-000000000001', 'a1000000-0000-7000-8000-000000000002');"

expect_fail "two active Tech Leads in one lab" \
  "BEGIN;
   INSERT INTO agents (id, lab_id, name, role, character_id, description, autonomy_level)
   VALUES ('a1000000-0000-7000-8000-000000000011', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaaa',
           'Lead One', 'tech_lead', 'veteran_lead',
           'Fixture Tech Lead with a sufficiently long description used only to exercise the one-active-Tech-Lead partial unique index in this test.', 2);
   INSERT INTO agents (id, lab_id, name, role, character_id, description, autonomy_level)
   VALUES ('a1000000-0000-7000-8000-000000000012', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaaa',
           'Lead Two', 'tech_lead', 'veteran_lead',
           'Second fixture Tech Lead that must be rejected by the one-active-Tech-Lead partial unique index in this test.', 2);
   COMMIT;"

expect_fail "invalid estimate 4" \
  "INSERT INTO cards (id, board_id, project_id, card_key, seq, title, column_name, priority, estimate)
   VALUES ('d0000000-0000-7000-8000-000000000004', 'cccccccc-cccc-7ccc-8ccc-cccccccccccc',
           'bbbbbbbb-bbbb-7bbb-8bbb-bbbbbbbbbbbb', 'TST-4', 4, 'Bad estimate', 'TODO', 'should', 4);"

expect_fail "ICEBOX without blocked reason" \
  "INSERT INTO cards (id, board_id, project_id, card_key, seq, title, column_name)
   VALUES ('d0000000-0000-7000-8000-000000000005', 'cccccccc-cccc-7ccc-8ccc-cccccccccccc',
           'bbbbbbbb-bbbb-7bbb-8bbb-bbbbbbbbbbbb', 'TST-5', 5, 'Bad ICEBOX', 'ICEBOX');"

expect_fail "project key_prefix must be uppercase letters/digits" \
  "INSERT INTO projects (id, lab_id, name, slug, key_prefix, source_ref)
   VALUES ('bbbbbbbb-bbbb-7bbb-8bbb-bbbbbbbbbbbc', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaaa',
           'Bad Prefix', 'bad-prefix', 'bad', 'example/bad');"

expect_fail "agent description shorter than 120 characters" \
  "INSERT INTO agents (id, lab_id, name, role, character_id, description, autonomy_level)
   VALUES ('a1000000-0000-7000-8000-000000000021', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaaa',
           'Short Desc', 'dev', 'scrappy_dev', 'too short', 2);"

expect_fail "duplicate (project_id, card_key)" \
  "INSERT INTO cards (id, board_id, project_id, card_key, seq, title, column_name)
   SELECT 'd0000000-0000-7000-8000-000000000006', board_id, project_id, card_key, 60, 'Dup key', 'TODO'
   FROM cards WHERE card_key = 'TST-100';"

expect_fail "duplicate (project_id, seq)" \
  "INSERT INTO cards (id, board_id, project_id, card_key, seq, title, column_name)
   SELECT 'd0000000-0000-7000-8000-000000000007', board_id, project_id, 'TST-7', seq, 'Dup seq', 'TODO'
   FROM cards WHERE card_key = 'TST-100';"

expect_fail "duplicate (provider_id, name)" \
  "INSERT INTO models (id, provider_id, name, display_name, context_window, max_output_tokens)
   VALUES ('f0000000-0000-7000-8000-000000000002', 'e0000000-0000-7000-8000-000000000001',
           'deepseek-flash', 'Duplicate', 128000, 8192);"

expect_fail "memory scope 'lab' with a project_id" \
  "INSERT INTO memory_documents (id, lab_id, project_id, scope, kind, title, content, content_hash, created_by)
   VALUES ('a2000000-0000-7000-8000-000000000002', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaaa',
           'bbbbbbbb-bbbb-7bbb-8bbb-bbbbbbbbbbbb', 'lab', 'note', 'Bad scope', 'x', 'hash-2', 'system');"

expect_fail "duplicate (lab_id, seq) in domain_events" \
  "INSERT INTO domain_events (id, lab_id, seq, event_type, aggregate_type, aggregate_id, actor_type, payload)
   VALUES ('e1000000-0000-7000-8000-000000000001', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaaa', 1, 'test', 'card', 'd0000000-0000-7000-8000-000000000010', 'system', '{}'),
          ('e1000000-0000-7000-8000-000000000002', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaaa', 1, 'test', 'card', 'd0000000-0000-7000-8000-000000000010', 'system', '{}');"

expect_fail "FK cards.project_id references a missing project" \
  "INSERT INTO cards (id, board_id, project_id, card_key, seq, title, column_name)
   VALUES ('d0000000-0000-7000-8000-000000000008', 'cccccccc-cccc-7ccc-8ccc-cccccccccccc',
           'bbbbbbbb-bbbb-7bbb-8bbb-bbbbbbbbbbbc', 'TST-8', 8, 'Bad FK', 'TODO');"

expect_fail "FK memory_chunks.document_id references a missing document" \
  "INSERT INTO memory_chunks (id, document_id, lab_id, scope, ordinal, text, token_count, embedding)
   VALUES ('a3000000-0000-7000-8000-000000000001', 'a2000000-0000-7000-8000-0000000000ff',
           'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaaa', 'lab', 1, 'x', 1, array_fill(0::real, ARRAY[384])::vector);"

expect_fail "memory chunk embedding must be 384 dimensions" \
  "INSERT INTO memory_chunks (id, document_id, lab_id, scope, ordinal, text, token_count, embedding)
   VALUES ('a3000000-0000-7000-8000-000000000002', 'a2000000-0000-7000-8000-000000000001',
           'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaaa', 'lab', 1, 'x', 1, array_fill(0::real, ARRAY[3])::vector);"

# Optimistic concurrency: the second update with version=1 must affect zero rows.
psql_c >/dev/null <<'SQL'
INSERT INTO cards (id, board_id, project_id, card_key, seq, title, column_name, priority, estimate)
VALUES ('d0000000-0000-7000-8000-000000000010', 'cccccccc-cccc-7ccc-8ccc-cccccccccccc',
        'bbbbbbbb-bbbb-7bbb-8bbb-bbbbbbbbbbbb', 'TST-10', 10, 'Concurrency card', 'TODO', 'should', 3);
SQL
first="$(psql_c -tAc "WITH u AS (UPDATE cards SET title='one', version=version+1 WHERE id='d0000000-0000-7000-8000-000000000010' AND version=1 RETURNING 1) SELECT count(*) FROM u")"
second="$(psql_c -tAc "WITH u AS (UPDATE cards SET title='two', version=version+1 WHERE id='d0000000-0000-7000-8000-000000000010' AND version=1 RETURNING 1) SELECT count(*) FROM u")"
if [ "$first" = "1" ] && [ "$second" = "0" ]; then
  printf 'PASS  optimistic concurrency (one winner)\n'; PASS=$((PASS + 1))
else
  printf 'FAIL  optimistic concurrency (first=%s second=%s)\n' "$first" "$second"; FAIL=1
fi

echo
if [ "$FAIL" -ne 0 ]; then
  echo "CONSTRAINTS: FAILED ($PASS passed)"
  exit 1
fi
echo "CONSTRAINTS: OK ($PASS cases)"
exit 0
