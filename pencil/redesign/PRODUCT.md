# Peti — product context

Source: read-only inspection of `/Users/bbbadry/Downloads/github-tools` on 15 October 2026. These are repository-derived facts, not new feature approvals. This package describes the proposed canvas redesign; it does not change the running app.

## Purpose and platform

Peti downloads public GitHub repository snapshots to a local folder. It runs in a Tauri 2 desktop shell with a TypeScript frontend and Rust core, targeting macOS, Windows and Linux. Snapshots do not include Git history or `.git`.

## Product boundaries

- Preserve the name Peti and `brand/logo/slot-symbol-v1.svg`.
- Five destinations: Home, Downloads, Discover, Library, Settings.
- Paste one or multiple repository URLs; invalid entries do not block valid entries.
- Select destination, queue downloads, inspect progress, pause, resume, cancel and retry.
- Show percent and ETA only when meaningful; otherwise show received bytes and speed. Explain when resume restarts the transfer.
- Keep completed items and their actual folders in Library. Never overwrite existing folders.
- Discover supports search, language, GitHub topic and period filters, details, direct download and batch selection. Popularity uses stars/forks; it is not a claim to reproduce GitHub Trending.
- GitHub connection is optional. Explicitly choose only download, only fork, or fork then download. Identify both fork and folder when applicable.
- Settings cover default folder, concurrency 1–8, system/light/dark theme and GitHub account.

## Source authority

`rencana/00-mulai-dari-sini.md`, `01-pengunduh.md`, `02-temukan.md`, `03-github.md`, `05-halaman-dan-alur.md`; current behavior is determined by `src/main.ts`, supporting TypeScript modules and `src-tauri/src`.

## Audience and open assumptions

Developers and people exploring open source are an inferred audience. Light is the provisional primary presentation, with a dark variant. The user requested a modern total redesign with Mobbin references and did not select a theme during creation. No pricing, analytics, collaboration, mobile app, private repository support or execution/build features are introduced.
