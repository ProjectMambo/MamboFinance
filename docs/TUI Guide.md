---
title: MamboFinance TUI guide
description: Run and navigate the current in-memory MamboFinance terminal prototype.
order: 20
---

::page{layout="docs" width="normal" sidebar=true}

# MamboFinance TUI guide

## Run

```bash
cargo run -p mambofinance-tui
```

The process opens an alternate terminal screen and restores the terminal on normal exit. Press `Ctrl+C` to quit.

> [!WARNING]
> The current TUI uses an in-memory database and seeded demonstration data. All changes disappear at exit.

## Views

The sidebar exposes five record tables:

1. Transactions
2. Groups
3. Categories
4. Funds
5. Currencies

Use `Up`/`Down` or `k`/`j` in the sidebar. Use `Right`/`l` to focus the table and `Left`/`h` to return. The table uses `Up`/`Down` or `k`/`j` for row selection.

## Add a record

Press `a` from the main panel to open the add form for the active table.

- Type into text fields; `Backspace` deletes one character and `Ctrl+Backspace` clears the field.
- Use arrow keys to move through horizontal or vertical option fields.
- Use `Tab` or `Enter` to advance and `Shift+Tab` to go back.
- Press `Enter` on the final row to submit.
- Press `Escape` to close the popup.

A transaction form collects name, description, integer amount, currency, day, month, year, group, category, and fund. Reference-data views collect their corresponding names; categories also select single or paired behavior.

Malformed amount input is silently coerced to `0`; malformed day, month, and year fields default to `1`, `1`, and `2000`, respectively. Missing selections become empty strings; downstream lookups and writes can surface as an application error or panic. This is prototype behavior, not a stable input contract.

## Unimplemented hints

The bottom bar currently advertises these keys without connected actions:

- `d` — delete
- `e` — edit
- `s` — sort
- `f` — filter

The library contains much of the underlying behavior, but the TUI handlers are not implemented. Do not interpret the visible hints as finished features.
