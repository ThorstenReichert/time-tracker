# ThorSoft TimeTracker

A personal time tracker for Windows that lives in the tray. It's built for billing work hours
to several clients: switching the running timer takes a hotkey and a few keystrokes, and the
daily overview shows the hours to book per client and project, rounded by each client's rules.

Single user, local only: the data stays in a SQLite file on your machine. No account, no
network, no telemetry.

## Features

- **Clients → projects → tasks.** Create a task by typing its name. Clients, projects and
  tasks can have short abbreviations.
- **Start, stop, switch.** At most one timer runs, and entries never overlap.
- **Quick-switch popup** on a global hotkey (default `Ctrl+Alt+Space`). Type to match a
  client/project/task combo, or pick one of the recent combos.
- **Tray icon**, single instance, starts with Windows.
- **Manual corrections:** add, edit and delete entries, fix a forgotten stop, add notes.
- **Daily overview:** the hours to book per project or per client, rounded by the client's
  rule (unit 6/12/15/30/60 min; up, nearest or down; at project or client level). The raw
  time and the difference are shown next to them. Click a value to copy it.
- **Backups:** a copy of the database at every start, in a folder you choose, keeping the
  newest N.
- **Light and dark mode**, following Windows.

What's planned and what's out of scope is in [spec.md](spec.md). What each version
contains is in [release-notes.md](release-notes.md).

## Install

There are no published binaries. Build the installer yourself (see below) and run
`ThorSoft TimeTracker_<version>_x64-setup.exe`. It installs per user and isn't signed, so
Windows SmartScreen may warn about it.

The database is `%LOCALAPPDATA%\ThorSoft.TimeTracker\timetracker.db`.

## Build

Prerequisites (Windows): Rust (the version pinned in `rust-toolchain.toml`, MSVC), Visual
Studio Build Tools with the C++ workload, Node 24 with npm, WebView2, and the Tauri CLI:

```
cargo install tauri-cli --version "^2.12" --locked
```

From the repository root:

```
npm --prefix ui ci                                            # once after cloning
pwsh scripts/check.ps1                                        # fmt, clippy, Rust and UI tests
cargo tauri dev --config src-tauri/tauri.dev.conf.json        # run a dev build
cargo tauri build                                             # build the NSIS installer
```

The dev build runs next to an installed copy, with its own identity and the hotkey
`Ctrl+Alt+Shift+Space`.

## How it's built

Rust (Tauri 2), Vue 3 with TypeScript and PrimeVue, SQLite. The domain logic is a pure,
deterministic core (a Decider: `decide` / `evolve`) in `crates/tt-core`. The UI, the hotkey
and the tests drive it through the same single write path. `crates/tt-cli` is a headless
driver for scenario tests. The design and the decisions behind it are in [spec.md](spec.md).
[CLAUDE.md](CLAUDE.md) gives an overview of the code.

Most of the code was written with an AI coding agent (Claude Code), following the process in
`.claude/skills/` (intent → spec → plan → build → review).

## License

[MIT](LICENSE)
