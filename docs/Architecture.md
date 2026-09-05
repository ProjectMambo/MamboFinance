---
title: MamboFinance Architecture
description: Workspace structure, SQLite data model, query layer, and current TUI boundary.
order: 10
---

::page{layout="docs" width="normal" sidebar=true}

# MamboFinance Architecture

## Workspace

```text
mambofinance-lib
    -> domain values and validation
    -> SQLite schema and operations
    -> typed query/filter/order/update API

mambofinance-tui
    -> terminal lifecycle and event loop
    -> UI and widget state
    -> calls mambofinance-lib
```

The workspace uses Rust 2024 edition. `mambofinance-tui` depends on the sibling library by path; the library has no dependency on the interface.

## Storage model

`User::new(name)` opens `storage/<name>.db`, creates the directory when needed, enables SQLite foreign keys, and creates missing tables. `User::new_in_memory(name)` applies the same schema to an in-memory connection.

The schema contains:

- `transactions`, including an integer amount, date parts, optional description, foreign keys, and an optional linked-transaction ID.
- `groups` for grouping ledger activity.
- `categories`, with a checked single or paired variant.
- `funds` for the account or store of value.
- `currencies` referenced by transactions.

UUID values are stored as SQLite blobs. Foreign keys cascade deletes to dependent transactions. Domain wrappers validate names, descriptions, amount representation, and calendar dates before writes.

## Ledger operations

The public `User` API creates and retrieves reference data, adds single or paired transactions, and exposes typed query objects. Query code supplies record-specific filtering and ordering while retaining a common wrapper for the UI.

Edit and delete operations exist in the library. Category variant changes protect linked transactions unless the caller explicitly uses the force path.

Budgets are not part of the active module tree. The empty budget placeholder does not provide a supported feature.

## TUI state flow

```text
Crossterm event
    -> App update loop
    -> UIState / active panel
    -> sidebar, query table, or popup
    -> UIEvent for a completed add form
    -> User write + table refresh
```

Ratatui renders a sidebar, the selected query table, a contextual bottom bar, and an add popup. Completed popup fields are compiled into strings and handed to the record-specific library call.

## Current prototype boundary

The binary creates `User::new_in_memory("USER")`, seeds currencies, funds, groups, categories, and transactions, and then starts the terminal loop. Nothing from that run is durable. Edit, delete, sort, and filter hints are visible but do not have key handlers yet.

Before a durable release, the interface still needs explicit user/database selection, input-error handling without panics, finished actions, migration/recovery policy, and an end-to-end persistence check.

## Quality gates

The library's unit tests cover domain validation, SQLite operations, relationships, and query behavior. Run:

```bash
cargo fmt --all -- --check
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets
```

The GitHub Actions workflow currently runs on `main`; active TUI work on other branches must be checked locally.
