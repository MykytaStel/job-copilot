# Design System Improvement Backlog

> Updated: 2026-04-22

This document is an implementation backlog for the current web design-system cleanup.
It is not an architecture ADR and does not change domain contracts.

## Goals

- make typography deterministic across environments
- remove global style leakage from primitives
- converge page surfaces on shared tokens
- normalize button states and contrast across pages
- add lightweight guardrails so cleanup does not regress

## Current Gaps

- typography depended on local font availability instead of repo-managed font delivery
- global `button` styling leaked primary visuals into places that use plain HTML buttons
- multiple page sections still used hardcoded `24px` / `28px` radii instead of design tokens
- some page styles still bypassed semantic color tokens with local hex values
- the current guard only validates token files, not enough usage conventions

## Next Slices

### 1. Primitive Hardening

- scope: `Button`, `Card`, `Badge`, input/select/textarea baseline, dialog shell surfaces
- acceptance:
  - no primary button visuals are defined in global reset CSS
  - primary, outline, ghost, link, and icon buttons have explicit contrast rules
  - card/dialog surfaces use shared radius tokens

### 2. Surface Normalization

- scope: profile, application detail, feedback center, market, dashboard shell surfaces
- current progress:
  - shared `SurfaceSection`, `SurfaceHero`, `SurfaceInset`, and `SurfaceMetric` primitives now back the main profile and application surfaces
  - shared `AccentIconFrame` now normalizes repeated accent-icon shells in shared section headers and core dashboard/market/settings surfaces
- acceptance:
  - main sections use `--radius-card` / `--radius-hero`
  - repeated card-like sections stop introducing local radius values
  - skeletons and loading shells visually match live surfaces

### 3. Semantic State System

- scope: fit score, lifecycle state, feedback state, notification state, badges/chips
- current progress:
  - shared semantic tone maps now back `Badge`, `StatusBadge`, `AIInsightPanel`, `PillList`, market trend badges, notification icons, and selected AI/job-detail callouts
  - repeated raw state classes are being replaced with shared `primary/info/success/warning/danger/muted` tone contracts
- acceptance:
  - success / warning / danger / accent tones come from semantic tokens
  - page CSS no longer carries ad hoc hex values for state text where tokens exist
  - button-adjacent state chips remain readable on dark surfaces

### 4. Typography System

- scope: heading/body/eyebrow scales, sidebar/header density, empty states
- acceptance:
  - font delivery is repo-managed
  - heading sizes are intentional and consistent by surface level
  - small labels keep readable contrast and spacing on dark UI

### 5. Guardrails And Verification

- scope: `guard-design-system`, UI smoke tests, lint-facing conventions
- acceptance:
  - hardcoded radius regressions are detectable
  - basic DS-critical paths remain covered by `build`, `test`, `lint`, and `guard:styles`
  - docs point contributors to tokens first, local CSS overrides second
