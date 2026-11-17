---
title: AI import workflow
description: Stage external-AI proposals, resolve deterministic conflicts, preview effects, and commit safely.
order: 30
---

::page{layout="docs" width="normal" sidebar=true}

# AI import workflow

## Chosen approach

MamboFinance provides finance tools; it does not choose or embed the model. A user may upload a report to ChatGPT or another capable system, and that system calls MamboFinance through its MCP server. The model performs document reading and uncertain interpretation while MamboFinance performs deterministic ledger work.

This separation makes the model replaceable, keeps finance rules testable, and lets the application mark uncertainty instead of inventing certainty.

## Host adapters

MCP is the portable AI boundary. For example, ChatGPT currently connects custom MCP servers as plugins with read and write tools, including private connectivity through [Secure MCP Tunnel](https://developers.openai.com/api/docs/guides/custom-mcp-server).

The server marks `get_import_context`, `list_import_batches`, `get_import_batch`, and `preview_import` as read-only. Staging, patching, abandoning, and committing are write operations, even when they affect only review state. Host confirmation is useful defense in depth, but the core still enforces validation, exact-target checks, preview freshness, authorization, and atomic commit. An optional CLI may mirror the same operations for scripts and debugging; it is not the AI integration boundary.

## Batch lifecycle

```text
staged -> needs_review -> ready -> committed
   |            |           |
   +----------> abandoned <-+
```

A committed batch is immutable. Corrections use ordinary ledger edits with an audit link or a new import; retrying the same committed source must return an idempotent result rather than duplicate its entries.

## Tool sequence

The MCP tool names below describe the intended contract and are not implemented yet.

| Step | Tool operation | Responsibility |
|---|---|---|
| 1 | `get_import_context` | Return the import schema plus bounded accounts, categories, configured mappings, currencies, and supported issue codes |
| 2 | `stage_import` | Accept source metadata and normalized proposed rows, create a batch, and run deterministic checks |
| 3 | `get_import_batch` | List staged rows, decisions, evidence locators, and issues with stable IDs and pagination |
| 4 | `patch_import` | Apply explicit row corrections, mappings, transfer links, accept decisions, or exclusions, then rerun affected checks |
| 5 | `preview_import` | Report inserts, skips, exclusions, conflicts, account effects, and statement-balance reconciliation without mutation |
| 6 | `commit_import` | Atomically create the approved ledger entries and provenance links, refusing unresolved rows |

The server also exposes `list_import_batches` and `abandon_import`. It never exposes raw SQL, arbitrary file access, or a generic “run command” escape hatch.

## Proposal input

Each batch identifies the source institution or marketplace, account when known, statement period, original filename or user-facing label, and a SHA-256 fingerprint when the host can calculate one. A fingerprint is an identifier for idempotency, not proof that extracted content is correct.

Each proposed row includes:

- a stable source row locator such as page and row, record index, or provider transaction ID;
- posting or transaction date with the source meaning preserved;
- signed integer minor units and explicit currency;
- original payee or description text plus an optional normalized payee;
- proposed account, category, transfer link, and note when justified;
- extraction confidence or explanation as evidence only, never as a validation bypass.

The core rejects malformed dates, amounts, currencies, missing source locators, unknown identifiers, oversized fields, and duplicate row IDs before mutation. It does not accept floating-point money.

## Deterministic checks

Staging and each patch rerun the applicable checks:

- exact source fingerprint or provider-ID reuse;
- exact transaction duplicate candidates using stable source identity;
- likely duplicates using account, date, amount, currency, and normalized description;
- missing or ambiguous account and category mappings;
- transfer candidates across owned accounts, including amount or currency differences;
- statement opening/closing balance or subtotal mismatch when the source provides totals;
- contradictory rows, unsupported currencies, and invalid dates or amounts.

An exact duplicate may be proposed as a skip. A likely duplicate, transfer candidate, ambiguous mapping, or balance mismatch remains an issue until explicitly resolved or excluded. The model may explain and patch a proposal, but it may not lower the core's checks.

## Review and commit

Every row has a visible decision such as `ready`, `skip_exact_duplicate`, `needs_review`, or `excluded`, plus zero or more stable issue codes. Exclusion requires a reason. A batch becomes ready only when no row is unresolved and its declared reconciliation checks pass or carry an explicit reviewed exception.

Preview is side-effect-free and returns the exact planned inserts, skips, exclusions, transfer links, and balance effects. Commit revalidates the preview assumptions inside one SQLite transaction; concurrent changes or new conflicts invalidate the preview and require another review. Partial imports are not reported as success.

## Privacy and retention

The preferred flow leaves the raw report with the user-selected AI host and sends MamboFinance only normalized proposals and bounded evidence. If a future local parser accepts a file, raw retention must be opt-in with a documented deletion path.

Context responses disclose only what the current import needs. Logs and errors use batch and row identifiers instead of full descriptions, account numbers, or report bodies. Tool clients must receive the same validation, confirmation, and authorization behavior as the human interface.

## Model failure behavior

Missing rows, invented merchants, swapped signs, misread dates, and unjustified categories are expected failure modes. MamboFinance therefore treats confidence as advisory, preserves the original source text and row locator, compares statement totals when available, and keeps unresolved proposals out of the ledger. A user can finish a batch manually without the original model remaining available.
