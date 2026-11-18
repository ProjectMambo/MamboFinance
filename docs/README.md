# MamboFinance

<p align="left">
  <img src="https://img.shields.io/badge/Rust-1.95.0-000000?style=flat-square&logo=rust&logoColor=white" alt="Rust 1.95.0" />
  <img src="https://img.shields.io/badge/SQLite-Foundation-003B57?style=flat-square&logo=sqlite&logoColor=white" alt="SQLite foundation implemented" />
  <img src="https://img.shields.io/badge/MamboUI-0.2.0-7a5fff?style=flat-square" alt="MamboUI 0.2.0" />
</p>
<p align="left">
  <a href="https://github.com/ProjectMambo/MamboFinance/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ProjectMambo/MamboFinance/ci.yml?style=flat-square&label=CI" alt="CI status" /></a>
  <img src="https://img.shields.io/badge/Status-Experimental-yellow?style=flat-square" alt="Project status: experimental" />
  <img src="https://img.shields.io/github/last-commit/ProjectMambo/MamboFinance?style=flat-square&color=7a5fff" alt="Last commit" />
  <img src="https://img.shields.io/github/repo-size/ProjectMambo/MamboFinance?style=flat-square&color=yellow" alt="Repository size" />
  <img src="https://img.shields.io/badge/License-AGPLv3_%2B_Commercial-orange?style=flat-square" alt="License: AGPLv3 and commercial" />
</p>

MamboFinance is an experimental local-first personal finance application for transactions, budgets, overviews, statistics, and reviewable AI-assisted statement imports.

## Motivation

People should be able to understand and control their financial ledger without manually retyping every bank or marketplace statement. MamboFinance keeps deterministic finance rules and durable data local while allowing a user-selected external AI to extract and propose entries through a narrow, reviewable tool contract.

## Status

MamboFinance now has its first account slice: one Rust package, a MamboUI terminal shell with account creation and listing under Manage, SQLite schema version 2 with durable accounts, and an authenticated loopback Streamable HTTP MCP server exposing `get_capabilities`, `list_accounts`, and `create_account`. Account editing and archiving, categories, transactions, budgets, statistics, imports, backup, and recovery are not implemented.

The obsolete library and in-memory prototype remain removed, and the replacement does not preserve their API or schema. The current `0.1.0` manifest is experimental and has no released compatibility promise. Version 1 will not embed or train a model.

## User stories

- As a user, I can manage accounts, categories, transactions, transfers, and budgets in a local ledger.
- As a user, I can see balances, budget progress, spending statistics, and items that still need review.
- As a user, I can give a bank or marketplace report to an AI of my choice, review uncertain rows, and commit only a validated import.
- As an AI client, I can perform every ledger and review action through typed tools with explicit read/write behavior and structured results.
- As a data owner, I can trace an imported entry to its source without silently retaining or uploading the original report.

## First scope

The first useful release covers accounts, categories, signed minor-unit transactions, linked transfers, category budgets, derived overview/statistics queries, MCP tools for every domain action, and staged imports with provenance and conflict review. Tags, groups, recurring rules, exchange-rate history, split transactions, cloud sync, and an embedded model are deferred until a demonstrated need justifies them.

## Dependencies

| Dependency | Classification | Purpose | Provider, version pin, or source | Scope | Update path |
|---|---|---|---|---|---|
| Rust toolchain | Tool | Build and test the application | `1.95.0` from `Cargo.toml` and CI | Build/test | Change the manifest and workflow together after the full gate passes |
| MamboUI | Sibling package | Shared Ratatui presentation primitives and compatible Ratatui types | Exact Git revision `357f6c55bb5e0b8a843ae12437cfe39646abbaab`, crate `0.2.0` | Runtime | Review the provider diff, update the exact revision, and run render plus interaction checks |
| rusqlite | Package and embedded database | SQLite connections, pragmas, and migrations | `0.43`, bundled SQLite, locked in `Cargo.lock` | Runtime | Update through a tested migration and storage gate |
| RMCP | Package | Typed MCP tools and Streamable HTTP service | `3.11`, locked in `Cargo.lock` | Runtime | Review protocol and schema changes, then repeat the end-to-end MCP smoke test |
| Axum and Tokio | Packages | Loopback HTTP serving and asynchronous runtime | `axum 0.8`, `tokio 1`, locked in `Cargo.lock` | Runtime | Update the lockfile and repeat server lifecycle checks |
| Crossterm | Package | Terminal setup, restoration, and keyboard input | `0.28`, locked in `Cargo.lock` | Runtime | Update with MamboUI-compatible render and terminal checks |
| Serde and Schemars | Packages | Structured tool results and JSON Schema | Major version `1`, locked in `Cargo.lock` | Runtime | Review generated MCP schemas before updating |
| tempfile | Development package | Isolated real-SQLite tests | Major version `3`, locked in `Cargo.lock` | Test | Update with the complete test gate |

MamboUI re-exports its matching Ratatui version, so MamboFinance does not add a second Ratatui dependency. No model SDK is planned.

## Getting started

Install Rust 1.95.0, clone the repository, and run the experimental terminal shell:

```bash
cargo run --locked
```

Start the foundation MCP server separately:

```bash
export MAMBOFINANCE_MCP_TOKEN="replace-with-at-least-32-private-characters"
cargo run --locked -- mcp
```

It listens only at `http://127.0.0.1:3210/mcp`. Clients must send that token as `Authorization: Bearer <token>`; keep it out of URLs, logs, and shared shell files. The implemented tools are `get_capabilities`, `list_accounts`, and `create_account`. Account names are trimmed, internal whitespace is collapsed, normalized names must be unique, and currencies are three ASCII letters normalized to uppercase.

## Documentation

| Goal | Document |
|---|---|
| Read the published project hub | [projectmambo.org/mambofinance/](https://projectmambo.org/mambofinance/) |
| Understand the selected architecture and reset decision | [Architecture](Architecture.md) |
| Follow the dependency-ordered implementation stages | [Development plan](Development%20Plan.md) |
| Understand the planned human and AI-tool workflows | [User guide](User%20Guide.md) |
| Understand external-AI staging, review, and commit | [AI import workflow](AI%20Import%20Workflow.md) |

## Project structure

```text
README.md              project entry point
Cargo.toml             Rust package and direct dependency contract
Cargo.lock             exact application dependency graph
src/                   shared application, SQLite, TUI, and MCP code
docs/                  synchronized product and development documentation
.github/workflows/     locked-toolchain validation
LICENSE                 repository licence summary
LICENSE-AGPL            AGPLv3 terms
LICENSE-COMMERCIAL.pdf  draft commercial terms
```

`target/` and local SQLite files are ignored build or runtime output. Production data uses the platform user-data directory rather than the repository.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
git diff --check
git status --short
```

From the canonical notes vault, maintainers also run `node Scripts/sync_docs.js --sync MamboFinance MamboWiki` and review every generated addition, modification, and deletion.

## Development

Build one vertical path at a time through the shared application core. The MamboUI interface, MCP server, and any optional CLI must call the same operations and validation; presentation and transport code must not own finance rules, and an AI proposal must never write directly to the ledger. Account creation and listing now demonstrate this adapter parity; the remaining account lifecycle must extend the same operations rather than adding adapter-specific rules.

Author documentation in `notes/Docs/Projects/MamboFinance/`, synchronize it, and update the documented gate when executable code returns. External pull requests are not currently requested; focused bug and design reports are welcome as repository issues.

## License

MamboFinance is dual-licensed under the GNU AGPLv3 and a commercial license:

- [LICENSE-AGPL](../LICENSE-AGPL)
- [LICENSE-COMMERCIAL](../LICENSE-COMMERCIAL.pdf) — draft, for informational purposes only
