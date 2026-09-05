---
title: MamboFinance
description: An experimental Rust and SQLite finance ledger with a Ratatui interface.
order: 40
---

::page{layout="project" width="normal" sidebar=true}

# MamboFinance

MamboFinance explores a local-first finance ledger built from a tested SQLite library and a terminal interface. The domain layer is substantially implemented; the TUI is still a demo-backed prototype.

::button{label="Source code" href="https://github.com/ProjectMambo/MamboFinance" variant="secondary" external=true}

## Current boundary

- The library persists named users to `storage/<name>.db` or can run entirely in memory.
- Transactions can reference currencies, groups, categories, and funds; paired categories support linked double entries.
- Query operations support filtering, ordering, editing, and deletion in the library.
- The current binary uses an in-memory `USER`, inserts demonstration data on every launch, and supports navigation and adding records.
- Budgets and the TUI's advertised edit, delete, sort, and filter actions are not implemented yet.

## Documentation

::children{view="list" sort="order" direction="asc" show=["title","description"]}

## Project status

The `tui` branch is active development. Use it for evaluation only and do not rely on the current interface for durable financial data.
