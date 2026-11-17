---
title: Development plan
description: Dependency-ordered stages for delivering the MamboFinance ledger, MamboUI interface, statistics, storage, and AI tools.
order: 15
---

::page{layout="docs" width="normal" sidebar=true}

# Development plan

This plan delivers MamboFinance as a local-first Rust application with a MamboUI terminal interface, durable SQLite storage, and an MCP server for external AI clients. It is ordered by dependency and safety: each stage leaves a complete, testable path rather than a collection of disconnected frameworks.

The plan is authoritative for intended delivery order, but it is not a promise of calendar dates. Update it when implementation evidence changes a domain rule, public interface, stage boundary, or release gate.

## Target shape

~~~text
MamboUI TUI -------\
                    > application use cases -> domain rules -> SqliteStore -> SQLite
MCP server --------/
~~~

Use one Rust package and one executable. The default mode starts the TUI; an mcp mode starts the foreground MCP server. Both adapters call the same use cases and validation. A complete finance CLI, generic repository layer, daemon, event bus, plugin system, model abstraction, or second storage implementation is not part of version 1.

Every meaningful domain action is complete only when it has:

1. an explicit domain rule and failure behavior;
2. SQLite storage and any required migration;
3. one shared application command or query;
4. a human workflow where applicable;
5. a typed MCP tool where it is a domain action;
6. focused tests, fixtures, and current documentation.

Navigation, focus, theme, and help remain presentation-only and do not need tool equivalents.

## Version 1 boundaries

Version 1 includes accounts, categories, signed minor-unit transactions, transfer links, explicit-period category budgets, derived overview and statistics queries, staged imports, provenance, typed MCP parity, verified backup and export, and safe concurrent TUI/MCP access.

The first version deliberately excludes split transactions, tags, recurring rules, investment accounting, direct bank connections, embedded OCR or models, cloud sync, multi-user access, automatic exchange rates, encrypted database formats, materialized statistics, a mapping-rule language, and an always-running daemon. Simple exact mappings may be added only after import testing shows repeated manual work.

## Stage 0: Freeze contracts and fixtures

Settle the rules that would otherwise force repeated migrations:

- Money is checked signed 64-bit minor units internally and never floating point.
- MCP serializes minor units as decimal strings so JSON clients cannot lose integer precision.
- Currency is explicit and an ordinary transaction matches its account currency.
- Dates are calendar dates; audit timestamps use UTC.
- Inflows are positive, outflows are negative, and opening balances are explicit adjustment transactions.
- Transfers link two transactions and do not count as income, expense, or budget spending.
- Cross-currency transfers keep both real amounts without inventing a rate.
- Referenced accounts and categories are archived rather than cascade-deleted.
- Transactions are the ledger source of truth; overview and statistics remain derived queries.
- Import proposals never affect the ledger before atomic commit.
- Raw statements are not retained by default.
- SQLite integer identifiers remain internal stable IDs and cross MCP as strings.
- Refund, uncategorized-flow, budget-overlap, and deletion semantics are fixed with examples before schema work.

Create one golden fixture containing two currencies, income, spending, a refund, an adjustment, an uncategorized entry, same- and cross-currency transfers, a budget, an exact duplicate, a possible duplicate, an ambiguous transfer, and a statement-balance mismatch.

Exit gate: balances, cash flow, budget use, statistics, and import outcomes have agreed hand-calculated results.

## Stage 1: Executable spine and MCP connection spike

Create the smallest working package with domain, application, storage, TUI, and MCP modules. Add the locked minimum Rust version, exact MamboUI revision, formatting, linting, tests, and dependency/licence checks. Production startup must not seed demonstration data.

The TUI initially renders an empty MamboUI shell. The MCP mode exposes only a harmless get_capabilities tool with the contract version, supported features, and limits.

Start with a loopback streaming HTTP transport. Add stdio only when a tested host needs it. Before selecting a permanent Rust MCP library, prove discovery, authentication, tool invocation, payload limits, timeouts, and host confirmation through one real external host and, when practical, a second MCP client. ChatGPT can connect read and write MCP tools through a custom server, and a private local endpoint can remain behind a [Secure MCP Tunnel](https://developers.openai.com/api/docs/guides/secure-mcp-tunnels).

Exit gate: a clean environment can start the server and call get_capabilities end to end through the chosen connection.

## Stage 2: Durable SQLite foundation

Implement one concrete SqliteStore and test it with real temporary SQLite databases. Do not build an in-memory persistence implementation that can drift from production.

The storage foundation must:

- use the platform user-data directory rather than the current directory;
- create a private application directory where the platform permits it;
- enable foreign keys on every connection;
- use WAL and a finite busy timeout for concurrent TUI and MCP processes;
- record ordered schema migrations and apply each migration transactionally;
- reject a database newer than the running binary;
- support rollback through a validated pre-upgrade backup rather than downgrade migrations;
- keep write transactions short and return a stable busy error;
- reserve multi-row writes before changing state;
- use optimistic revisions for stale edits and previews.

The first schema grows into these tables:

| Table | Responsibility |
|---|---|
| accounts | Name, kind, currency, and archive state |
| categories | Classification and archive state |
| transactions | Ledger source of truth and provenance |
| transfers | Links between two transaction sides |
| budgets | Category, period, currency, and limit |
| audit_events | Interface, operation, target, request ID, and timestamp |
| import_batches | Source metadata and review lifecycle |
| import_rows | Proposal, evidence locator, decision, and committed result |
| import_issues | Stable deterministic conflicts and allowed resolutions |
| schema_migrations | Applied schema versions |

Exit gate: empty bootstrap, restart, repeat migration, failed-migration rollback, newer-schema refusal, two-connection access, and basic consistent backup all pass.

## Stage 3: Accounts, categories, and first-run path

Deliver the first complete vertical slice.

Accounts cover bank, card, cash, wallet, and marketplace balances, with one currency and an active or archived state. Categories begin as a flat optional classification. Names use deliberate normalized uniqueness, and referenced records can be archived and restored but not destructively removed.

The TUI gains five-tab navigation, a Manage menu, account and category screens, inline validation, and an empty state that leads directly to first-account creation. Rejected input remains in the editor.

The MCP surface gains list, get, create, update, archive, and restore tools for accounts and categories.

Exit gate: records survive restart, archive preserves history, and TUI and MCP return the same results and structured errors.

## Stage 4: Transactions and transfers

Transactions store account, posting date, optional transaction date, amount, currency, payee, optional preserved source description, optional category and note, origin, revision, and audit metadata. Origin distinguishes manual, MCP, import, and adjustment entries without placing sensitive payloads in logs.

The Transactions tab provides:

- a date, payee, account, category, amount, and currency table;
- filters for date, account, category, currency, uncategorized state, origin, and transfer state;
- a detail pane on wide terminals and a separate detail screen on narrow terminals;
- add, edit, categorize, provenance, transfer, and deletion-preview workflows;
- exact-target confirmations that explain transfer, budget, and provenance effects.

A transfer links exactly two distinct transactions. Each side belongs to at most one transfer. Same-currency transfers require opposite equal amounts; cross-currency transfers require opposite signs while preserving each amount. Link, unlink, and creation operations are atomic.

MCP adds bounded transaction list/get/create/update, preview-delete and delete operations, plus link-transfer and unlink-transfer. Updates and deletes require expected revisions; creates accept idempotency keys.

Exit gate: no partial transfer can survive, stale edits fail without data loss, and filters and selection remain valid after mutation.

## Stage 5: Budgets

A budget applies one non-negative minor-unit limit to one category, inclusive date range, and currency. Overlapping budgets for the same category and currency are rejected until the product defines unambiguous combination behavior. Transfers and balance adjustments never consume a budget.

The Budgets tab shows period and currency selectors plus limit, spent, remaining or overage, and textual states such as OK, NEAR LIMIT, and OVER. Version 1 has no recurring rule engine. An explicit copy-to-next-period action may be added only when users demonstrate the need.

MCP adds list, get, progress, create, update, and delete budget tools.

Exit gate: budget results reconcile against the golden fixture and update immediately after transaction changes.

## Stage 6: Overview and Insights

Separate immediate status from exploratory analysis.

The Overview tab answers where the user stands now:

- balances grouped by currency;
- current-period income, expense, and net cash flow;
- budget progress and overruns;
- import batches needing review;
- recent transactions.

Each summary opens its filtered destination. Empty data points to the next useful action.

The Insights tab accepts explicit date, currency, account, and category filters. Initial metrics are account balance and period change, income, expense, net cash flow, category breakdown, payee breakdown, daily or weekly or monthly trends, budget variance, and comparison with the preceding equivalent period. Ratios such as savings rate are omitted when their denominator is not meaningful.

Never combine currencies into one amount. Exclude transfers and balance adjustments from cash-flow measures, show exact comparison dates, and begin with text and ranked tables. Add native Ratatui charts only when they communicate more than the table.

Calculate statistics at query time. Add indexes after inspecting realistic query plans; introduce summary tables only after measurement proves they are needed.

MCP adds get_overview and a bounded get_statistics tool with finite metric, grouping, period, account, category, and currency options rather than an unrestricted query language.

Exit gate: every displayed value reconciles to known fixtures and representative 10,000- and 100,000-transaction ledgers remain responsive.

## Stage 7: Import staging and deterministic review

Add the import staging tables and issue engine before enabling ledger commit.

~~~text
staged -> needs_review -> ready -> committed
   |            |           |
   +----------> abandoned <-+
~~~

Staging checks exact source reuse, provider identifiers, likely duplicates, missing or ambiguous accounts, unknown proposed categories, transfer candidates, invalid dates and amounts, repeated row locators, field and batch limits, and declared balance mismatches.

Safe deterministic handling may apply an exact user-approved mapping, skip an exact source duplicate, or link a transfer only when the match is unique and all invariants pass. Possible duplicates, ambiguous mappings, unsupported values, and balance exceptions remain visible until explicitly resolved.

MCP adds get_import_context, list_import_batches, get_import_batch, stage_import, patch_import, and abandon_import. Patch operations use typed row actions and an expected batch revision rather than arbitrary JSON Patch.

The Imports tab gains a batch list, row/evidence review, issue filters, corrections, duplicate and transfer decisions, and exclusion with a reason. Confidence remains evidence and never grants approval.

Exit gate: staging never changes ledger totals, review survives restart, and every fixture produces stable issue codes and allowed resolutions.

## Stage 8: Preview, AI resolution, and atomic commit

Complete the external-AI sequence:

1. The client requests bounded import context.
2. The user-selected AI reads the report and stages normalized proposals with source locators.
3. MamboFinance runs deterministic checks.
4. The client patches issues it can justify.
5. The user reviews remaining uncertainty in Imports.
6. Preview returns exact inserts, skips, exclusions, transfer links, balance effects, reconciliation results, blockers, and a short-lived freshness token.
7. Commit revalidates and applies the complete plan in one SQLite transaction.

Commit requires the preview token, expected batch revision, idempotency key, import-commit authorization, no unresolved rows, and unchanged ledger assumptions. Repeated commits return the prior result instead of duplicating entries. A committed batch is immutable.

Exit gate: stage through MCP, review in the TUI, commit through MCP, and observe correct Overview and Insights changes with row-level provenance.

## Stage 9: Security, backup, export, and recovery

The MCP server remains disabled until explicitly enabled, binds only to loopback, and uses revocable credentials. Initial scopes are ledger.read, ledger.write, import.review, import.commit, and data.backup. Import connections do not receive general ledger-write, commit, or backup permission by default.

Every tool has typed input and output schemas, bounded cursor pagination, stable errors, accurate read-only and destructive annotations, server-side validation, least-data responses, expected revisions, and idempotency where required. The server exposes no raw SQL, shell, arbitrary file path, generic command, or unrestricted ledger dump.

Data operations include:

- online backup through SQLite's backup API or VACUUM INTO, never a raw copy of live WAL files;
- integrity and schema checks before reporting success;
- a lossless versioned JSON export;
- a CSV transaction export for interoperability, not restoration;
- restore into a temporary database, validation, backup of current data, and atomic replacement under exclusive ownership;
- automatic pre-migration backup once real user schemas evolve.

Migrations are startup and release behavior rather than AI tools. Restore remains an operator/TUI-only version 1 recovery action because it replaces the entire ledger. Any future restore tool needs a separate preview and commit, an exact backup identifier, exclusive locking, and destructive confirmation.

Exit gate: a clean profile restored from backup reproduces rows, relationships, provenance, and statistics; hostile report text stays inert; logs contain no credentials, reports, account numbers, or complete descriptions.

## Stage 10: Hardening and version 1 release

Run the complete gate:

- unit tests for money, dates, transfers, budgets, and import decisions;
- SQLite integration tests for every application use case;
- migrations from every released schema;
- MCP schema snapshots and a tool-parity inventory;
- every authorization scope and annotation;
- concurrent TUI and MCP updates, stale revisions, and database-busy behavior;
- injected mid-commit failure with complete rollback;
- backup and restore round trips;
- hostile or prompt-like source descriptions treated as inert data;
- log and fixture privacy review;
- light, dark, monochrome, wide, 80-by-24, and reduced-terminal render checks;
- keyboard-only transaction and import review;
- terminal restoration on normal exit and failure;
- clean installation and packaging on every supported platform;
- synchronized user, developer, and tool documentation.

The version 1 acceptance journey creates accounts and categories, records and edits transactions, links a transfer, creates a budget, verifies Overview and Insights, stages and resolves a report through MCP, commits it, verifies the same results through both interfaces, backs up the ledger, and restores identical results into a clean profile.

## Interface map

Use five primary tabs and keep infrequent administration in Manage:

| Area | Purpose |
|---|---|
| Overview | Current balances, flow, budgets, recent activity, and review counts |
| Transactions | Search, edit, categorize, transfer, and inspect provenance |
| Budgets | Set limits and compare actual spending |
| Insights | Explore filtered statistics and period comparisons |
| Imports | Review batches, resolve conflicts, preview, and commit |
| Manage | Accounts, categories, data safety, MCP status, appearance, and help |

Global navigation uses direct tab keys, focus movement, contextual search, a help overlay, the Manage menu, and quit confirmation only when work is uncommitted. Define 80 by 24 as the initial full-use target and ensure smaller terminals fail gracefully or use a reduced single-pane layout.

Finance-owned UI utilities are added only when their first screen needs them: money and date formatting, table scrolling, filter summaries, field validation, exact-target confirmation, issue labels, progress text, provenance display, and genuinely slow-operation state. Do not add a generic form framework, message bus, command palette, widget trait, background task framework, or chart abstraction.

## MCP tool map

Deliver tools with each domain slice rather than postponing tool parity:

| Area | Read tools | Write tools |
|---|---|---|
| System | get_capabilities, get_database_status | create_backup when authorized |
| Accounts | list_accounts, get_account | create, update, archive, and restore account |
| Categories | list_categories, get_category | create, update, archive, and restore category |
| Transactions | list_transactions, get_transaction, preview_delete_transaction | create, update, and delete transaction |
| Transfers | transfer details through transaction reads | link_transfer and unlink_transfer |
| Budgets | list_budgets, get_budget, get_budget_progress | create, update, and delete budget |
| Reporting | get_overview and get_statistics | None |
| Imports | context, list, get, and preview | stage, patch, abandon, and commit |

Common conventions are decimal-string minor units, ISO dates, RFC 3339 timestamps, bounded cursor pagination, deterministic ordering, expected revisions, idempotency keys, resulting-resource responses, and stable codes such as validation_failed, not_found, conflict, stale_revision, unresolved_issues, unauthorized, forbidden, and database_busy.

## MamboUI workstream

The current MamboUI primitives are enough to begin. MamboFinance composes Shell, Panel, selection tables, StatusLine, Help, EmptyState, Theme, centered overlays, and native Ratatui widgets. Finance retains navigation, focus, forms, business validation, charts, and commands.

Implement likely gaps locally first:

| Need | First implementation |
|---|---|
| Primary tabs | Native Ratatui Tabs styled with Theme |
| Forms and field errors | Finance-owned state and rendering |
| Confirmation overlays | centered_rect, Clear, and Panel |
| Scrolling | Ratatui Scrollbar and Finance selection state |
| Unequal master/detail layouts | Local Ratatui Layout |
| Budget progress | Text or native Gauge |
| Insights charts | Native Ratatui widgets |
| Large shortcut sets | Short footer plus Finance help overlay |

A primitive moves into MamboUI only after a second real application needs the same presentation rule. Promotion includes the smallest shared implementation, a render test, demo or example, public documentation, a SemVer decision, migration notes where needed, a reviewed MamboFinance pin update, and Finance regression checks. MamboUI never owns Finance state, persistence, domain rules, or MCP behavior.

## Milestones

| Milestone | Included stages | Outcome |
|---|---|---|
| Internal foundation | 0 through 2 | Proven architecture, connection, migrations, and durable store |
| 0.1 | 3 and 4 plus validated backup | Useful manual ledger with transfers |
| 0.2 | 5 and 6 | Budgets, overview, and statistics |
| 0.3 | 7 and 8 | Reviewable external-AI import workflow |
| 0.9 | 9 | Recovery, privacy, authorization, and concurrency hardening |
| 1.0 | 10 | Complete acceptance journey and supported migration contract |

No milestone advances while an earlier exit gate remains open. If implementation evidence changes a rule, update the fixture, architecture, plan, and affected public contracts together rather than preserving a stale stage description.
