---
phase: 01
slug: codebase-health-rebrand
status: verified
threats_open: 0
asvs_level: 1
created: 2026-04-21
---

# Phase 01 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Icon assets → Build | Generated icon files included in app bundle | Static assets, no sensitive data |
| Config → Build | Tauri config values are trusted build inputs | Bundle identifier, product name |
| Docker → Deployment | Container names and image names affect network identity | Service naming |
| Package registry → Build | Renamed npm scope may conflict with existing packages | Package names only |
| Import resolution → Runtime | Module split and re-exports must resolve correctly | Code module boundaries |
| Type re-exports → Type system | Barrel re-exports must cover all existing types | TypeScript types |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-01-01 | S | Icon assets | accept | Placeholder icons for dev use only — professional design swaps later | closed |
| T-01-02 | T | globals.css | accept | Color tokens are cosmetic only, no security impact | closed |
| T-02-01 | S | Bundle identifier | mitigate | Verified com.whaleit.app doesn't conflict, build passes | closed |
| T-02-02 | I | Deep-link schemes | accept | Scheme change from whaleit:// to whaleit:// breaks existing deep links — expected during rebrand | closed |
| T-02-03 | T | Auth salt strings | mitigate | Explicitly preserved auth.rs salt strings unchanged during execution | closed |
| T-03-01 | S | Package scope | accept | @whaleit/* scope verified by pnpm install success | closed |
| T-03-02 | D | Import paths | mitigate | Build + type-check + 505 tests verify all imports resolve | closed |
| T-04-01 | D | Type imports | mitigate | type-check + 505 tests verify all barrel re-exports work | closed |
| T-04-02 | D | Web adapter dispatch | mitigate | Build + 505 tests verify all dispatch cases preserved | closed |

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-01 | T-01-01 | Placeholder whale icon is geometric SVG — professional illustration to replace later | Phase executor | 2026-04-20 |
| AR-02 | T-01-02 | Color palette tokens are cosmetic, zero security surface | Phase executor | 2026-04-20 |
| AR-03 | T-02-02 | Deep-link scheme change breaks existing whaleit:// links — expected and documented in rebrand | Phase executor | 2026-04-20 |
| AR-04 | T-03-01 | @whaleit/* npm scope unlikely to conflict with real packages | Phase executor | 2026-04-20 |

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-04-21 | 9 | 9 | 0 | gsd-security-audit |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-04-21
