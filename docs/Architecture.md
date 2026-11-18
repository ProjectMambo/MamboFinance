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

The implemented schema version 2 records ordered migration history and active or archived accounts. An account has an internal integer ID, normalized unique name key, original display name, constrained kind, three-letter uppercase currency, revision, UTC timestamps, and optional archive timestamp. Startup opens the database in the platform user-data directory, creates the application directory with private Unix permissions, enables foreign keys and WAL, applies a five-second busy timeout, applies each missing migration in its own immediate transaction, and rejects newer schemas. Categories, ledger tables, failed-upgrade recovery, and the backup contract remain later stages.

### MamboUI terminal interface

The TUI pins MamboUI revision `357f6c55bb5e0b8a843ae12437cfe39646abbaab` at crate version `0.2.0`. It imports Ratatui through `mambo_ui::ratatui` and composes MamboUI's shell, panels, status lines, help, empty states, and native Ratatui tabs. Finance-specific screens, forms, charts, conflict evidence, and state transitions stay in MamboFinance.

The five primary screens are Overview, Transactions, Budgets, Insights, and Imports. Infrequent account, category, data-safety, MCP-status, appearance, and help tasks live under Manage rather than crowding the daily navigation. The implemented Manage account screen lists active accounts and provides a keyboard-only create form whose rejected input remains editable. Rendering and input handling call application commands and do not contain validation or database policy.

### MCP tools and optional CLI

Every domain query or mutation available in the TUI has a typed MCP tool. Navigation, focus, theme, and help are presentation-only and need no tool equivalent. Each tool has explicit input and output schemas, stable structured errors, and an accurate read or write annotation.

The MCP server exposes bounded finance operations rather than unrestricted SQL, shell, or filesystem access. Read tools return only the data required for the task. Write tools preserve authorization and exact-target checks, require confirmation for consequential actions, and use preview, idempotency, or atomic mutation where required.

An optional `mbfinance` CLI may wrap the same application operations for scripts and debugging. It is not required for AI connectivity and cannot weaken MCP or core validation.

The implemented MCP foundation runs as `mambofinance mcp` at the fixed loopback endpoint `http://127.0.0.1:3210/mcp`. It requires `MAMBOFINANCE_MCP_TOKEN` at startup and rejects requests whose bearer header does not exactly match it. The token must contain at least 32 ASCII letters, digits, hyphens, or underscores. The official Rust MCP SDK exposes structured `get_capabilities`, `list_accounts`, and `create_account` tools with accurate read-only, destructive, and idempotent annotations. Validation and conflict failures use stable JSON error content. Revocation UI, multiple scoped credentials, configurable connection settings, request limits, and the remaining finance tools are unavailable.

## Trust boundaries

- The local user and MamboFinance process control the ledger database.
- A user-selected AI host may see reports that the user uploads to it; that disclosure is outside MamboFinance and must remain explicit.
- MamboFinance returns only the accounts, categories, configured mappings, schema, and issue details needed for the current import, not an unrestricted transaction history.
- AI output is untrusted proposal data. It is validated like any other external input and never writes directly to ledger tables.
- Unresolved or conflicting rows remain visible until a user or authorized client resolves or explicitly excludes them.

## Delivery order

The [development plan](Development%20Plan.md) owns the detailed dependency order, exit gates, interface map, storage contract, MamboUI workstream, MCP tool map, and release milestones. In summary, the implementation establishes the domain, migrations, connection boundary, and focused checks before delivering each domain slice through both MamboUI and typed MCP tools. Durable backup precedes user-data claims, and staged import builds on the completed ledger and reporting core.

The reset deliberately leaves no placeholder crates, plugin system, model abstraction, or cloud service.
