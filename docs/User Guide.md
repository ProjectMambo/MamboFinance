---
title: User guide
description: Planned manual, budget, overview, AI-tool, and import-review workflows for the rebuilt application.
order: 20
---

::page{layout="docs" width="normal" sidebar=true}

# User guide

> [!WARNING]
> The executable foundation is available, but the finance workflows below remain planned. The current TUI only demonstrates the five-view shell and storage status; the current MCP server only exposes `get_capabilities`.

## Foundation preview

Run the TUI from a clone with Rust 1.95.0:

```bash
cargo run --locked
```

Use `1` through `5` or the left and right arrow keys to inspect the planned views, and `q` to quit. The shell intentionally contains no account or ledger actions yet.

Start the local MCP endpoint with:

```bash
cargo run --locked -- mcp
```

It binds only to `127.0.0.1:3210`, serves MCP at `/mcp`, and reports implemented and unavailable capabilities. Do not treat it as a complete finance integration or expose it to another network.

## Human interface

The MamboUI terminal interface will keep five primary views, with infrequent administration under Manage:

| View | Purpose |
|---|---|
| Overview | Show account balances, budget progress, recent spending summaries, and the count of imports needing review |
| Transactions | Add, edit, categorize, link as transfers, filter, inspect provenance, and remove entries with confirmation |
| Budgets | Set category limits for a date period and compare actual spending with the limit |
| Insights | Explore income, expense, cash-flow, category, payee, balance, and budget statistics with explicit filters and currency boundaries |
| Imports | Inspect batches and evidence, edit proposals, accept or exclude rows, resolve conflicts, preview effects, and commit |

Manage contains accounts, categories, backup/export/restore, database status, MCP connection state, appearance, and the complete key reference. It is not a daily tab.

Every status has a text label rather than colour alone. The interface shows active keys, confirms destructive actions with the exact target, preserves unresolved work, and remains usable at its documented minimum terminal size.

## Manual ledger workflow

1. Create an account with its real currency.
2. Create only the categories needed for budgets or review.
3. Add a transaction using signed minor units, date, payee, and account; category and note remain optional.
4. Link the two sides of an account-to-account movement as a transfer rather than categorizing them as spending and income.
5. Review the overview and statistics, which are calculated from committed transactions.

Referenced accounts and categories are archived or protected instead of silently deleting their transactions.

## Budgets and reviews

A budget applies one category limit to one date period and currency. The overview compares committed spending with that limit and does not convert between currencies in version 1. Statistics are derived views, so editing a transaction updates subsequent results without maintaining a second aggregate store.

## AI-assisted statement import

1. Give a bank, card, wallet, or marketplace report to an external AI you trust.
2. Let the AI request the bounded MamboFinance import context and submit normalized row proposals with source locators.
3. MamboFinance stages the batch and reports exact duplicates, possible duplicates, missing mappings, transfer candidates, invalid rows, and balance mismatches.
4. Let the AI patch issues it can justify, then inspect anything that remains unresolved in the Imports view.
5. Preview inserts, skips, exclusions, conflicts, and balance effects.
6. Commit the ready batch atomically. Unresolved rows must be resolved or explicitly excluded; they are never silently guessed.

See [AI import workflow](AI%20Import%20Workflow.md) for the full trust and tool contract.

## AI tool parity

MamboFinance exposes typed tools through MCP. Exact tool names become public only when implemented and tested, but the interface covers:

| Area | Required tool operations |
|---|---|
| Overview and statistics | Read summaries with explicit periods, account/category filters, and currency boundaries |
| Accounts and categories | List, show, create, update, archive, and restore |
| Transactions and transfers | List, show, add, update, link/unlink, preview deletion, and delete |
| Budgets | List, show, create, update, and delete |
| Imports | Read context; stage, inspect, patch, review, preview, commit, and abandon a batch |
| Data safety | Export, create an application-owned backup, and inspect database status |

The MamboUI interface and MCP tools call the same application operations. UI-only navigation, focus, theme selection, and help do not need tool equivalents. Tools declare typed inputs and outputs, stable errors, and read or write behavior; they never expose a generic shell, SQL, or filesystem operation.

Migrations run through the supported application startup and release path rather than an AI tool. Restore remains an operator-only recovery workflow in version 1 because it replaces the complete ledger. An optional CLI may wrap ordinary operations for scripts or debugging, but it is not required for AI connectivity.

## Data and privacy

The ledger will live in the platform's user data directory. Raw statements are not retained by default; imported transactions keep enough source identity and row-level provenance to explain and deduplicate them. Users must be able to export and back up the ledger before the application claims durable readiness.

An external AI sees only what the user gives that provider and the bounded context returned by MamboFinance. Users should remove irrelevant pages or identifiers before upload when the provider does not need them.
