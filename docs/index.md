---
title: MamboFinance
description: An experimental local-first finance app with MamboUI, typed AI tools, and reviewable external-AI imports.
order: 40
---

::page{layout="project" width="normal" sidebar=true}

# MamboFinance

MamboFinance is being rebuilt as a local-first ledger for transactions, transfers, budgets, overviews, statistics, and reviewable AI-assisted imports. It keeps finance rules and SQLite persistence deterministic while letting users choose an external AI to interpret bank and marketplace reports.

::button{label="Source code" href="https://github.com/ProjectMambo/MamboFinance" variant="secondary" external=true}

## Current status

The first account slice is available: the MamboUI Manage screen and bearer-authenticated MCP tools create and list durable SQLite accounts through the same validation. Account editing and archiving, transactions, budgets, statistics, imports, backup, and recovery remain planned; there is no stable data-compatibility promise yet.

## Selected direction

- One Rust package owns a shared application core and local SQLite ledger.
- A thin terminal interface uses MamboUI; an MCP server exposes every domain action as a typed AI tool.
- An optional `mbfinance` CLI may wrap the same operations for scripts and debugging.
- Imports are staged, validated, reviewed, previewed, and committed atomically; AI proposals never bypass ledger rules.
- Version 1 does not embed a model or retain raw reports by default.

## Documentation

::children{view="list" sort="order" direction="asc" show=["title","description"]}
