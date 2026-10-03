# MamboFinance

<p align="left">
  <img src="https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/SQLite-003B57?style=flat-square&logo=sqlite&logoColor=white" alt="SQLite" />
  <img src="https://img.shields.io/badge/Ratatui-FA5A5A?style=flat-square&logo=ratatui&logoColor=white" alt="Ratatui" />
</p>
<p align="left">
  <img src="https://img.shields.io/badge/Status-Prototype-yellow?style=flat-square" alt="Project status: prototype" />
  <img src="https://img.shields.io/github/last-commit/ProjectMambo/MamboFinance?style=flat-square&color=7a5fff" alt="Last commit" />
  <img src="https://img.shields.io/github/repo-size/ProjectMambo/MamboFinance?style=flat-square&color=yellow" alt="Repository size" />
  <img src="https://img.shields.io/badge/License-AGPLv3_%2B_Commercial-orange?style=flat-square" alt="License: AGPLv3 and commercial" />
</p>

MamboFinance is an experimental local finance ledger written in Rust. Its library models SQLite-backed transactions, categories, groups, funds, currencies, and queries; its Ratatui interface is currently a development prototype over an in-memory demo database.

## Motivation

Personal finance data benefits from a local, inspectable ledger with explicit domain rules instead of a service account or opaque remote store. MamboFinance explores that boundary with a reusable Rust library and a keyboard-driven terminal interface.

## Status

MamboFinance is a prototype under active development on the `tui` branch. The library has broad unit coverage; the binary still uses seeded in-memory data and is not a durable finance application. There is no packaged release or supported database-migration path.

## User stories

- As a library user, I can create validated reference data and single or paired ledger transactions in SQLite.
- As a terminal user, I can navigate record tables and add records without malformed fields crashing the process.
- As a local-data owner, I can use a named database without letting its name escape or redirect the repository-owned `storage/` boundary.
- As a maintainer, I can validate domain, query, transaction, and interface behavior through one documented workspace gate.

## Getting Started

Install a Rust toolchain with Rust 2024 edition support, then run the workspace from the repository root:

```bash
git clone https://github.com/ProjectMambo/MamboFinance.git
cd MamboFinance
cargo run -p mambofinance-tui
```

The SQLite dependency uses a bundled SQLite build, so a system SQLite development package is not required. The current process opens a seeded in-memory ledger and discards all changes on exit.

## Documentation

| Goal | Document |
|---|---|
| Read the canonical Wiki documentation | [projectmambo.org/mambofinance/](https://projectmambo.org/mambofinance/) |
| Understand storage and code boundaries | [Architecture](Architecture.md) |
| Run and operate the current prototype | [TUI guide](TUI%20Guide.md) |
| Inspect the source | [`mambofinance-lib/`](../mambofinance-lib/) and [`mambofinance-tui/`](../mambofinance-tui/) |

## Current capabilities

| Area | Implemented now |
|---|---|
| Library | File-backed or in-memory SQLite initialization; transactions and paired transactions; categories, groups, funds, and currencies; query, sort, filter, edit, and delete operations |
| TUI | Seeded in-memory session; table and sidebar navigation; views for each record type; add forms for transactions and reference data |
| Not yet wired in the TUI | Persistent user databases, edit, delete, sort, filter, budgets, import/export, and production error presentation |

The current interface discards its data when the process exits. The persistent `User::new` library path exists, but the binary intentionally calls `User::new_in_memory` and seeds demonstration records at startup.

Persistent library users must pass one non-empty filename component to `User::new`; path separators, absolute paths, `.` and `..` are rejected. The `storage/` directory and selected database must not be symbolic links. No migration, backup, encryption, or recovery contract exists yet, so keep independent backups and do not use this prototype as the only copy of financial records.

## Project Structure

```text
mambofinance-lib/       SQLite ledger, domain types, validation, and queries
mambofinance-tui/       Ratatui application, widgets, input, and event loop
.cargo/config.toml      workspace command aliases
.github/workflows/      Rust checks for the main and active TUI branches
docs/                   project, architecture, and TUI documentation
```

## Validation

```bash
cargo fmt --all -- --check
cargo test-all
cargo clippy --workspace --all-targets
git diff --check
git status --short
```

`cargo test-all` expands to `cargo test --workspace --no-fail-fast`. The workspace currently has comprehensive library tests; strict warning-free Clippy remains follow-up work because prototype and placeholder paths are intentionally unused.

## Development

Treat the public `mambofinance-lib` types, SQLite schema, validation behavior, and persistent path rules as interfaces. Coordinate incompatible changes with the TUI, add migration guidance before opening durable databases from the binary, and keep multi-statement ledger mutations transactional.

Author documentation in `notes/Docs/Projects/MamboFinance/`, then run `node Scripts/sync_docs.js --sync MamboFinance MamboWiki` from `notes/`. Review the synchronized README and `docs/` tree before committing.

This is a personal finance project, so external pull requests are not currently requested. Focused bug reports are welcome as repository issues.

## License

MamboFinance is dual-licensed under the GNU AGPLv3 and a commercial license:

- [LICENSE-AGPL](../LICENSE-AGPL)
- [LICENSE-COMMERCIAL](../LICENSE-COMMERCIAL.pdf) — draft, for informational purposes only
