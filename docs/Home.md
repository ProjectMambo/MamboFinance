---
description: A lightweight, privacy-focused financial dashboard for effortless expense tracking and budgeting.
tags:
  - Rust
  - SQLite
date: June 2026 - Present
wikiUrl:
githubUrl: https://github.com/ProjectMambo/MamboFinance
---
## Repo Structure

## Project Structure
```text
main
├── user
│   ├── transaction
│   ├── group
│   │   └── budget
│   ├── fund
│   ├── category
│   ├── types
│   │   ├── date
│   │   ├── amount
│   │   ├── currency
│   │   ├── label
│   │   └── pool
│   └── stats
│       └── ...
├── parse
│   ├── import
│   └── export
├── cli
│   └── ...
└── gui
    └── ...
```