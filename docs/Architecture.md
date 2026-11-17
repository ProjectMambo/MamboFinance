---
title: Architecture
description: The local ledger, shared command core, MamboUI and MCP interfaces, and external-AI trust boundary.
order: 10
---

::page{layout="docs" width="normal" sidebar=true}

# Architecture

## Decision

MamboFinance will not embed a model in version 1. Using an external AI is optional for the user, but the application exposes a complete typed tool interface for AI clients. Those clients may extract and propose statement rows; MamboFinance owns deterministic validation, conflict detection, review state, and persistence. This keeps the application model-independent and avoids making a cloud model a hidden trust boundary for financial data.

The replacement will be one Rust package with one application command layer and three thin adapters:

```text
Human -> MamboUI TUI -----------\
AI clients -> MCP server --------> application commands -> domain rules -> SQLite
Scripts -> optional CLI --------/
```

The MCP server is the canonical AI integration. It maps typed tools to the same application commands used by the TUI; it is not a second implementation. An optional CLI may wrap those operations for scripts and debugging. No adapter has a privileged mutation path.

## Why the old design was reset

The old code provided a large bespoke query and widget framework around a terminal demo that always seeded an in-memory database and discarded changes at exit. It had no machine-callable tool interface, import workflow, source provenance, migration contract, durable application path, statistics, or usable budgets, and it duplicated components now owned by MamboUI.

Its current-directory-relative storage and cascading reference-data deletes were unsafe foundations for real financial records. Tiny text limits, title-casing, and paired-category semantics also did not map cleanly to ordinary statements. Preserving that API and schema would cost more than rebuilding the small domain the product actually needs.

## Layers

### Domain and application core

The core owns finance invariants and use cases without terminal, MCP, or rendering types. Each command accepts validated input, executes one transaction where mutation is involved, and returns the same structured result or stable error to every adapter.

| Concept | First contract |
|---|---|
| `Account` | A bank, card, cash, wallet, or marketplace balance with a currency and active/archive state |
| `Category` | Optional classification used by transactions and budgets; referenced categories are archived or restricted rather than cascade-deleted |
| `Transaction` | Signed integer minor units, ISO currency, date, payee, optional note/category, account, and source provenance |
| `Transfer` | A link between two transactions, including different amounts or currencies, without inventing paired categories |
| `Budget` | A category, date period, currency, and minor-unit limit |
| `ImportBatch` | Source identity, fingerprint when available, statement metadata, lifecycle state, totals, and commit record |
| `ImportRow` | Original row locator and evidence, normalized proposal, issues, decision, and resulting transaction link |

Overview balances, budget progress, statistics, and review counts are queries derived from the ledger. They are not duplicate stored aggregates.

### Storage

SQLite is the local system of record. The application will use the platform-appropriate user data directory rather than the current working directory, enable foreign keys, and apply ordered transactionally recorded migrations. Money remains integer minor units and currency is explicit; version 1 does not perform exchange-rate conversion.

Reference data that is still in use cannot silently erase transactions. Multi-row imports and transfer creation commit atomically. Before the first durable release, the project must define backup/export, interrupted-migration recovery, database ownership, and supported downgrade behavior.

Source provenance is retained with imported entries, but raw statements are not copied into the database by default. Logs, JSON output, errors, and test fixtures must not expose full reports, account numbers, or unrelated ledger history.

### MamboUI terminal interface

The TUI will pin MamboUI revision `819b03fe77a39ee1bccefde4de9b9676eaa03134` until its `0.2.0` contract is released. It imports Ratatui through `mambo_ui::ratatui` and composes MamboUI's shell, panels, selection tables, status lines, help, empty states, and responsive layouts. Finance-specific screens, forms, charts, conflict evidence, and state transitions stay in MamboFinance.

The first screens are Overview, Transactions, Budgets, and Imports. Rendering and input handling call application commands and do not contain validation or database policy.

### MCP tools and optional CLI

Every domain query or mutation available in the TUI has a typed MCP tool. Navigation, focus, theme, and help are presentation-only and need no tool equivalent. Each tool has explicit input and output schemas, stable structured errors, and an accurate read or write annotation.

The MCP server exposes bounded finance operations rather than unrestricted SQL, shell, or filesystem access. Read tools return only the data required for the task. Write tools preserve authorization and exact-target checks, require confirmation for consequential actions, and use preview, idempotency, or atomic mutation where required.

An optional `mbfinance` CLI may wrap the same application operations for scripts and debugging. It is not required for AI connectivity and cannot weaken MCP or core validation.

## Trust boundaries

- The local user and MamboFinance process control the ledger database.
- A user-selected AI host may see reports that the user uploads to it; that disclosure is outside MamboFinance and must remain explicit.
- MamboFinance returns only the accounts, categories, configured mappings, schema, and issue details needed for the current import, not an unrestricted transaction history.
- AI output is untrusted proposal data. It is validated like any other external input and never writes directly to ledger tables.
- Unresolved or conflicting rows remain visible until a user or authorized client resolves or explicitly excludes them.

## Delivery order

1. Establish the domain, migrations, shared application operations, MCP transport and authorization boundary, and focused checks.
2. Deliver each domain slice through both MamboUI and typed MCP tools, with parity and failure-path tests.
3. Add overview, budgets, statistics, backup/export, and the same tool coverage.
4. Add staged import, deterministic reconciliation, review, preview, and atomic commit through the shared core and MCP tools.

The reset deliberately leaves no placeholder crates, plugin system, model abstraction, or cloud service.
