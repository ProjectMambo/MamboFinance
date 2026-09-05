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

## Start here

| Goal | Document |
|---|---|
| Read the canonical Wiki documentation | [projectmambo.org/mambofinance/](https://projectmambo.org/mambofinance/) |
| Understand storage and code boundaries | [Architecture](docs/Architecture.md) |
| Run and operate the current prototype | [TUI Guide](docs/TUI%20Guide.md) |
| Inspect the source | [`mambofinance-lib/`](mambofinance-lib/) and [`mambofinance-tui/`](mambofinance-tui/) |

## Current capabilities

| Area | Implemented now |
|---|---|
| Library | File-backed or in-memory SQLite initialization; transactions and paired transactions; categories, groups, funds, and currencies; query, sort, filter, edit, and delete operations |
| TUI | Seeded in-memory session; table and sidebar navigation; views for each record type; add forms for transactions and reference data |
| Not yet wired in the TUI | Persistent user databases, edit, delete, sort, filter, budgets, import/export, and production error presentation |

The current interface discards its data when the process exits. The persistent `User::new` library path exists, but the binary intentionally calls `User::new_in_memory` and seeds demonstration records at startup.

## Local setup

Install a Rust toolchain with Rust 2024 edition support, then run the workspace from the repository root:

```bash
git clone https://github.com/ProjectMambo/MamboFinance.git
cd MamboFinance
cargo run -p mambofinance-tui
```

The SQLite dependency uses a bundled SQLite build, so a system SQLite development package is not required.

## Repository layout

```text
mambofinance-lib/       SQLite ledger, domain types, validation, and queries
mambofinance-tui/       Ratatui application, widgets, input, and event loop
.cargo/config.toml      workspace command aliases
.github/workflows/      Rust checks for the main and active TUI branches
docs/                   project, architecture, and TUI documentation
```

## Development checks

```bash
cargo fmt --all -- --check
cargo test-all
cargo clippy --workspace --all-targets
git diff --check
git status --short
```

`cargo test-all` expands to `cargo test --workspace --no-fail-fast`. The workspace currently has comprehensive library tests; strict warning-free Clippy remains follow-up work because prototype and placeholder paths are intentionally unused.

## Status

Development currently happens on the `tui` branch. CI runs the documented formatting, Clippy, and test sequence for both `main` and `tui`; there is no packaged release or installation command. Treat the binary as a prototype, not as the sole copy of financial records.

## Issues and feedback

This is a personal finance project, so external pull requests are not currently requested. Focused bug reports are welcome as repository issues.

## License

MamboFinance is dual-licensed under the GNU AGPLv3 and a commercial license:

- [LICENSE-AGPL](LICENSE-AGPL)
- [LICENSE-COMMERCIAL](LICENSE-COMMERCIAL.pdf) — draft, for informational purposes only
