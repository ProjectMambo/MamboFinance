---
title: MamboFinance
description: A local-first finance design with MamboUI, complete CLI parity, and reviewable external-AI imports.
order: 40
---

::page{layout="project" width="normal" sidebar=true}

# MamboFinance

MamboFinance is being rebuilt as a local-first ledger for transactions, transfers, budgets, overviews, statistics, and reviewable AI-assisted imports. It keeps finance rules and SQLite persistence deterministic while letting users choose an external AI to interpret bank and marketplace reports.

::button{label="Source code" href="https://github.com/ProjectMambo/MamboFinance" variant="secondary" external=true}

## Current status

The obsolete library and in-memory terminal prototype have been removed. The repository is now a documentation and licence baseline with no executable application or supported database. Git history remains available, but the replacement intentionally starts with a new schema and interface contract.

## Selected direction

- One Rust package owns a shared application core and local SQLite ledger.
- A thin terminal interface uses MamboUI; a non-interactive `mbfinance` CLI exposes every domain action with structured JSON.
- Optional MCP or other tool adapters map to the same commands for external assistants.
- Imports are staged, validated, reviewed, previewed, and committed atomically; AI proposals never bypass ledger rules.
- Version 1 does not embed a model or retain raw reports by default.

## Documentation

::children{view="list" sort="order" direction="asc" show=["title","description"]}
