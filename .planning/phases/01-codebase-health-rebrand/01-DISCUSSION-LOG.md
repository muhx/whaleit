# Phase 1: Codebase Health & Rebrand - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-20
**Phase:** 01-codebase-health-rebrand
**Areas discussed:** Rebrand boundary, Visual identity, Codebase health scope, Onboarding redesign

---

## Rebrand Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Keep @whaleit/* (Recommended) | Internal packages, not user-facing. Renaming npm scope means updating every import across 431 files with zero user benefit. | |
| Rename to @whaleit/* | Clean brand consistency. Every import changes. High churn, high risk. | ✓ |

**User's choice:** Rename to @whaleit/*

| Option | Description | Selected |
|--------|-------------|----------|
| Rename to connect/ | Rename dir and update route paths. Clean, consistent. | ✓ |
| Keep whaleit-connect/ | Internal path, not user-visible. Less risky. | |

**User's choice:** Rename to connect/

| Option | Description | Selected |
|--------|-------------|----------|
| Rename all user-visible Rust strings (Recommended) | Window title, menu items, about page, AI prompts → WhaleIt | ✓ |
| UI strings only | Only rename strings in UI components. Leave log messages, error types. | |

**User's choice:** Rename all user-visible Rust strings

| Option | Description | Selected |
|--------|-------------|----------|
| Update to com.whaleit.* (Recommended) | Change bundle ID for macOS/Windows. Clean brand. | ✓ |
| Keep com.teymz.whaleit | Avoids disrupting auto-update paths, keychain entries, code signing. | |

**User's choice:** Update to com.whaleit.*

| Option | Description | Selected |
|--------|-------------|----------|
| Full doc rebrand (Recommended) | README, Docker labels, compose services, GitHub metadata, all docs. | ✓ |
| App-facing docs only | Only rename app-facing docs. Leave developer docs as-is. | |

**User's choice:** Full doc rebrand

---

## Visual Identity

| Option | Description | Selected |
|--------|-------------|----------|
| Soft illustration style | Illustrated whale, soft curves, warm tones. Notion meets Duolingo. | ✓ |
| Minimal geometric | Clean geometric whale, flat design. Stripe/Linear aesthetic. | |
| Friendly cartoon | Playful cartoon whale, more expression. Slack mascot style. | |

**User's choice:** Soft illustration style

| Option | Description | Selected |
|--------|-------------|----------|
| Evolve existing palette (Recommended) | Keep warm paper tones, shift accents. Less jarring for users. | |
| Fresh palette from scratch | Design new color system around whale brand. More distinctive. | ✓ |

**User's choice:** Fresh palette from scratch

| Option | Description | Selected |
|--------|-------------|----------|
| App icon only | Whale on icon only. UI stays clean with text branding. | |
| Icon + splash screen | Whale on icon and loading screen. Branding moment on launch. | |
| Icon + splash + header | Whale on icon, splash, and header/sidebar. Consistent presence. | ✓ |

**User's choice:** Icon + splash + header

| Option | Description | Selected |
|--------|-------------|----------|
| AI-generated placeholder (Recommended) | AI generates icons in chosen style. Good for dev, swap for pro later. | ✓ |
| Design brief only | Write prompts for designer. Include design task in plan. | |

**User's choice:** AI-generated placeholder

---

## Codebase Health Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Modular split (Recommended) | Split 184-case switch into domain modules. Switch dispatches to modules. | ✓ |
| Generate from schema | Generate mapping from shared schema/OpenAPI. Zero manual sync but big upfront work. | |
| Skip refactor | Leave as-is. Only rename references. | |

**User's choice:** Modular split

| Option | Description | Selected |
|--------|-------------|----------|
| Domain-based split (Recommended) | Split into types/account.ts, types/portfolio.ts etc. Barrel re-export. | ✓ |
| Co-locate with features | Move types into feature directories. More file moves. | |
| Skip split | Leave as-is. Only remove deprecated types. | |

**User's choice:** Domain-based split

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, clean up deprecations | Remove ActivityLegacy and deprecated refs. Low risk. | ✓ |
| Skip, not blocking | Leave deprecated types. Not blocking, just messy. | |

**User's choice:** Yes, clean up deprecations

---

## Onboarding Redesign

| Option | Description | Selected |
|--------|-------------|----------|
| Rebrand only (Recommended) | Swap branding text, update colors/logo, keep existing step structure. | ✓ |
| Full redesign | Redesign onboarding flow with new steps, copy, visuals. | |
| Add welcome screen | Keep existing flow, add new WhaleIt welcome/splash screen. | |

**User's choice:** Rebrand only

---

## OpenCode's Discretion

- Exact domain groupings for types.ts split
- Exact module boundaries for web adapter split
- Splash/loading screen implementation approach
- Header whale brand element size and placement
- Specific color values for new palette
- Transition strategy for barrel re-exports during types.ts split

## Deferred Ideas

None — discussion stayed within phase scope.
