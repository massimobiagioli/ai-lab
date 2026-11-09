# WP-20 Restore Drill Evidence

| Field | Value |
|---|---|
| Date | 2026-11-09 |
| Dump | `artifacts/backups/20261109T133835Z.dump` |
| Drill | `scripts/restore-drill.sh artifacts/backups/20261109T133835Z.dump` |
| RTO | 52 s (target ≤ 3600 s) |
| Integrity | `orphan_cards=0`, `outbox_stuck=0`, `ready_missing_dor=0`, `memory_chunk_nulls=0`, `done_without_merge_sha=0`, `review_without_reviewer=0`, `card_projection_mismatch=0` |
| Report | `artifacts/backups/20261109T133835Z-restore-drill.json` (gitignored artifact) |
| Result | Row counts, content checksums and integrity verified; scratch DB dropped |
