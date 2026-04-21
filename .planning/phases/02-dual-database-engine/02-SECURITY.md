---
phase: 02
slug: dual-database-engine
status: verified
threats_open: 0
asvs_level: 1
created: 2026-04-21
---

# Phase 02 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Config → Diesel | DATABASE_URL from env var crosses into connection setup | PG connection string (credentials) |
| App → PostgreSQL | Database connection with credentials in DATABASE_URL | All financial data |
| Client → SQL queries | Repository method parameters via Diesel parameterized queries | Account IDs, dates, symbols |
| Docker network → PostgreSQL | Internal network, PG credentials in compose env | PG password |
| CI → PostgreSQL | Ephemeral test database in CI | Test-only data |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-02-01 | I (Info Disclosure) | Cargo.toml / config | mitigate | Connection strings in env vars only, never hardcoded. Verified: no `postgres://` literals in source. | closed |
| T-02-02a | S (Spoofing) | Repository traits | accept | Internal traits — no external input crosses this boundary | closed |
| T-02-02b | T (Tampering) | Repository queries | mitigate | All queries use Diesel query builder (parameterized). Raw SQL uses `$1`/`$2` bindings, `format!` only for compile-time constants (`SOURCE_PRIORITY_CASE`). Verified. | closed |
| T-02-03 | I (Info Disclosure) | DATABASE_URL | mitigate | PG credentials only in env var, never logged. Verified: no logging of connection strings in config or storage modules. | closed |
| T-02-04 | D (Denial of Service) | Connection pool | accept | Pool size configurable (8 default), reasonable for self-hosted | closed |
| T-02-05 | S (Spoofing) | PG connection | mitigate | PG password via `WF_PG_PASSWORD` env var with required validation (`${WF_PG_PASSWORD:?Set WF_PG_PASSWORD}`). Verified in compose.yml. | closed |
| T-02-06 | I (Info Disclosure) | Docker Compose | accept | PG port not exposed externally (internal Docker network only). Dev compose exposes on localhost only. | closed |
| T-02-07 | E (Elevation) | Feature flag | accept | `postgres` feature is compile-time (`cfg(feature = "postgres")`), not runtime — no dynamic switching attack surface | closed |
| T-02-08 | I (Info Disclosure) | CI test database | accept | Ephemeral test container, no production data, destroyed after run | closed |
| T-02-09 | S (Spoofing) | Test credentials | accept | Hardcoded test-only credentials in CI YAML — no production access | closed |
| T-02-05-01 | I | sync/app_sync.rs | accept | Sync stubs return errors for unsupported operations in PG mode — no data exposed | closed |
| T-02-05-02 | S | build_state_postgres | mitigate | Connection pool created from `std::env::var("DATABASE_URL")`, not user input. Verified in config.rs. | closed |
| T-02-06-01 | T (Tampering) | portfolio/snapshot | mitigate | All queries use Diesel parameterized DSL. Raw SQL uses `$1`/`$2` bindings. Verified. | closed |
| T-02-06-02 | I (Info Disclosure) | fx/market_data | mitigate | Symbol strings used via Diesel DSL bindings, not string interpolation. Verified. | closed |

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-01 | T-02-02a | Repository traits are internal — no external input boundary | opencode | 2026-04-21 |
| AR-02 | T-02-04 | Connection pool DoS — self-hosted, pool size configurable, reasonable default | opencode | 2026-04-21 |
| AR-03 | T-02-06 | PG port internal Docker network only — no external exposure in production | opencode | 2026-04-21 |
| AR-04 | T-02-07 | Compile-time feature flag — no runtime attack surface | opencode | 2026-04-21 |
| AR-05 | T-02-08 | CI ephemeral test database — no production data | opencode | 2026-04-21 |
| AR-06 | T-02-09 | Test-only hardcoded credentials — no production access | opencode | 2026-04-21 |
| AR-07 | T-02-05-01 | Sync stubs return errors in PG mode — sync not functional, no data leakage | opencode | 2026-04-21 |

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-04-21 | 14 | 14 | 0 | opencode |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-04-21
