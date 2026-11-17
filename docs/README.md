# MamboFinance

<p align="left">
  <img src="https://img.shields.io/badge/Rust-planned-000000?style=flat-square&logo=rust&logoColor=white" alt="Rust planned" />
  <img src="https://img.shields.io/badge/SQLite-planned-003B57?style=flat-square&logo=sqlite&logoColor=white" alt="SQLite planned" />
  <img src="https://img.shields.io/badge/MamboUI-planned-7a5fff?style=flat-square" alt="MamboUI planned" />
</p>
<p align="left">
  <img src="https://img.shields.io/badge/Status-Design_reset-yellow?style=flat-square" alt="Project status: design reset" />
  <img src="https://img.shields.io/github/last-commit/ProjectMambo/MamboFinance?style=flat-square&color=7a5fff" alt="Last commit" />
  <img src="https://img.shields.io/github/repo-size/ProjectMambo/MamboFinance?style=flat-square&color=yellow" alt="Repository size" />
  <img src="https://img.shields.io/badge/License-AGPLv3_%2B_Commercial-orange?style=flat-square" alt="License: AGPLv3 and commercial" />
</p>

MamboFinance is a local-first personal finance application design for transactions, budgets, overviews, statistics, and reviewable AI-assisted statement imports.

## Motivation

People should be able to understand and control their financial ledger without manually retyping every bank or marketplace statement. MamboFinance keeps deterministic finance rules and durable data local while allowing a user-selected external AI to extract and propose entries through a narrow, reviewable tool contract.

## Status

MamboFinance is at a clean design baseline. The obsolete Rust library and in-memory TUI prototype have been intentionally removed; there is currently no executable application, supported database, package, or migration path. Git history retains the old implementation for reference, but the replacement will not preserve its public API or schema.

The first implementation will be one Rust package with a shared application core, a MamboUI terminal interface, and an MCP server exposing a complete typed AI tool interface. An optional `mbfinance` CLI may wrap the same operations for scripts and debugging. Version 1 will not embed or train a model.

## User stories

- As a user, I can manage accounts, categories, transactions, transfers, and budgets in a local ledger.
- As a user, I can see balances, budget progress, spending statistics, and items that still need review.
- As a user, I can give a bank or marketplace report to an AI of my choice, review uncertain rows, and commit only a validated import.
- As an AI client, I can perform every ledger and review action through typed tools with explicit read/write behavior and structured results.
- As a data owner, I can trace an imported entry to its source without silently retaining or uploading the original report.

## First scope

The first useful release covers accounts, categories, signed minor-unit transactions, linked transfers, category budgets, derived overview/statistics queries, MCP tools for every domain action, and staged imports with provenance and conflict review. Tags, groups, recurring rules, exchange-rate history, split transactions, cloud sync, and an embedded model are deferred until a demonstrated need justifies them.

## Planned dependencies

No runtime or build dependency is active during the design reset. The implementation is expected to adopt only these foundations:

| Dependency | Classification | Purpose | Planned source | Scope | Update path |
|---|---|---|---|---|---|
| Rust toolchain | Tool | Build the application and its interfaces | Minimum version to be fixed in the first manifest | Build/test | Change the manifest only after the full gate passes on the new minimum |
| SQLite | Embedded database | Store the local ledger, import staging state, and migrations | Rust package and version to be selected with the first storage slice | Runtime | Update schema and dependency through a tested migration |
| MamboUI | Sibling package | Provide shared Ratatui presentation primitives | Exact Git revision `819b03fe77a39ee1bccefde4de9b9676eaa03134` until `0.2.0` is released | TUI runtime | Review a newer commit, update the exact revision, and run render plus interaction checks |

MamboUI re-exports its matching Ratatui version; MamboFinance will import Ratatui types through that re-export instead of adding a second Ratatui dependency. No model SDK is planned.

## Getting started

There is currently nothing to install or run. Start with the design documents below; executable setup instructions will be added only with the first working vertical slice.

## Documentation

| Goal | Document |
|---|---|
| Read the published project hub | [projectmambo.org/mambofinance/](https://projectmambo.org/mambofinance/) |
| Understand the selected architecture and reset decision | [Architecture](Architecture.md) |
| Understand the planned human and AI-tool workflows | [User guide](User%20Guide.md) |
| Understand external-AI staging, review, and commit | [AI import workflow](AI%20Import%20Workflow.md) |

## Project structure

```text
README.md              project entry point
docs/                  synchronized design documentation
LICENSE                 repository licence summary
LICENSE-AGPL            AGPLv3 terms
LICENSE-COMMERCIAL.pdf  draft commercial terms
```

Implementation directories, manifests, lockfiles, and CI return only when a working slice needs them; the reset does not keep placeholder code.

## Validation

The current gate is documentation-only:

```bash
git diff --check
git status --short
```

From the canonical notes vault, maintainers also run `node Scripts/sync_docs.js --sync MamboFinance MamboWiki` and review every generated addition, modification, and deletion.

## Development

Build one vertical path at a time through the shared application core. The MamboUI interface, MCP server, and any optional CLI must call the same operations and validation; presentation and transport code must not own finance rules, and an AI proposal must never write directly to the ledger.

Author documentation in `notes/Docs/Projects/MamboFinance/`, synchronize it, and update the documented gate when executable code returns. External pull requests are not currently requested; focused bug and design reports are welcome as repository issues.

## License

MamboFinance is dual-licensed under the GNU AGPLv3 and a commercial license:

- [LICENSE-AGPL](../LICENSE-AGPL)
- [LICENSE-COMMERCIAL](../LICENSE-COMMERCIAL.pdf) — draft, for informational purposes only
