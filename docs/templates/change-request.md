# CR-{NNNN} — {Title}

| Field | Value |
|---|---|
| id | `CR-{NNNN}` |
| title | {imperative summary} |
| decision | `approved|rejected|deferred` |
| approver | {role, YYYY-MM-DD} |
| date | {YYYY-MM-DD} |
| linked_adr | {ADR-NNNN or —} |

## context

{Why the change is proposed now. Link the trigger: incident, review finding, dependency, schedule or operator request.}

## proposed_change

{Exactly what changes. Name files, contracts, WPs and cards.}

## impact_scope

{Requirements, WPs, cards and contracts affected. State what is explicitly not affected.}

## impact_schedule

{Milestones and dates affected, or "None".}

## impact_cost

{Budget and effort delta, or "None".}

## impact_risk

{Risks introduced or retired, with mitigation.}

## alternatives

1. {Option} — {why rejected}.

## decision

{`approved`, `rejected` or `deferred` with rationale. A deferred CR states the reopen condition.}

## approver

{Role and date. The approval matrix is in `../change-control.md` §3.}

## linked_adr

{ADR identifier or —. When the change is architectural, the ADR exists in the same change.}
