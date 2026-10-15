<div align="center">

<img src="src-tauri/icons/128x128.png" width="80" alt="Peti">

# Peti

**Download a public GitHub repository into a folder. One revision, as files, without Git history.**

Window app. Not a menu-bar extra.

<br/>

[![Build](https://github.com/BadryansahBangsawan/peti/actions/workflows/ci.yml/badge.svg)](https://github.com/BadryansahBangsawan/peti/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/BadryansahBangsawan/peti?style=flat-square)](https://github.com/BadryansahBangsawan/peti/releases)

<br/>

| | |
|---|---|
| Product | `Peti` |
| Bundle ID | `app.peti` |
| Version | `0.1.0` |
| Window | 1100×720, minimum 720×480 |

</div>

---

## What you get

| Piece | Behavior |
|---|---|
| **Paste** | One or more `github.com/owner/repo` links, including a trailing `.git` and `/tree/<ref>`. SSH links are rejected. |
| **Queue** | Pause, resume, cancel, retry. At most 3 downloads at once (limit 1–8). There is no stepper for that number in Settings. |
| **Discover** | Public search on GitHub: text, language, topic, and period. Stars, 20 results, cached for 60 seconds. Not signed in. |
| **Fork** | Optional. Fork a public repo, or fork then download the fork. Device Flow, scope `public_repo` only. No client secret. |
| **Library** | Finished folders that are still real directories. A missing folder or a symlink is hidden. The history row is not deleted. |
| **Language** | Ten languages. Default is Bahasa Indonesia. English (UK) uses the same text as English and is stored separately. |
| **Theme** | System, light, or dark. Applied immediately. |

Peti does not copy Git history, upload files, or store the access token in its database.

---

## Download

No release is published yet. The files below appear on [Releases](https://github.com/BadryansahBangsawan/peti/releases) after a `v*` tag is pushed. Until then, use [Build from source](#build-from-source).

Same GitHub Release, two files:

| File | Use |
|---|---|
| **`Peti_*.dmg`** | Open, drag **Peti** onto **Applications** |
| **`Peti.app.zip`** | Unzip, then move **Peti.app** to Applications |

A build on this Mac on 15 October 2026 wrote `Peti_0.1.0_aarch64.dmg` (4.27 MiB) and `Peti.app` (arm64, 8.83 MiB). The built `Info.plist` sets `LSMinimumSystemVersion` to 10.13. The binary is Apple Silicon only, so this disk image does not run on Intel. The release workflow runs the same `npm run tauri build` on `macos-latest`.

---

## Install

### Disk image

1. Download `Peti_*.dmg` from [Releases](https://github.com/BadryansahBangsawan/peti/releases).
2. Open it. Drag **Peti** onto **Applications**.
3. First open. This build has no Apple Developer ID and is not notarized:

```bash
xattr -cr /Applications/Peti.app
open /Applications/Peti.app
```

Still blocked: System Settings → Privacy & Security → Open Anyway.

### Homebrew

Not set up. A cask needs a published `Peti.app.zip` and its sha256. Neither exists until the first release.

---

## How to open

Peti is a normal window, with a Dock icon. Open **Peti** from Applications or Launchpad. The window title is Peti.

---

## Usage

1. Open Peti. The interface language starts as Bahasa Indonesia. Navigation stays English: Home, Downloads, Discover, Library, Settings.
2. Paste a link on Downloads. The first download asks for a destination folder. That folder is reused.
3. Peti writes `{owner}-{repo}` inside that folder. If the name exists, it tries `{owner}-{repo}-2` through `-1000`. It does not overwrite a folder that is already there.
4. Watch progress on Downloads. Pause keeps the bytes already saved. Resume only works while the row is paused. Retry only works after a failure.
5. Finished folders show in Library while the directory is still there.
6. Discover filters language, topic, and period (today, this week, this month). An unknown period is treated as this week.
7. Settings changes the theme, the language, and the destination folder. The language applies without reloading the window.

Example links that are accepted:

```text
https://github.com/octocat/Hello-World
http://www.github.com/octocat/Hello-World.git
github.com/octocat/Hello-World/tree/main
```

A query string or fragment is ignored. One `.git` suffix is stripped.

### GitHub login

Login is only needed to fork. Create a GitHub OAuth app that uses Device Flow, then paste its client id in Settings when the field is shown. Shape only, not a real id: `Iv1.contoh-bukan-rahasia`.

You can also set it for one launch without writing the database:

```bash
PETI_GITHUB_CLIENT_ID='Iv1.contoh-bukan-rahasia' npm run tauri dev
```

Peti opens `https://github.com/login/device` and shows a code. The requested scope is `public_repo`. There is no client secret in the app.

From a Discover result: download, add to the queue, or fork. Your own repository is not forked.

Log out in Settings before you delete the app if you want the saved login removed.

---

## Data

| What | Where |
|---|---|
| Queue, history, folder, theme, language, client id | `~/Library/Application Support/app.peti/peti.db` on macOS |
| GitHub token and login | OS keyring, service `app.peti` |
| Downloaded files | The folder you picked. Not inside the app. |

The database has no token column. Windows and Linux data paths were not checked on this machine. **Needs confirmation** for those two.

---

## Privacy

Downloads and Discover talk to GitHub (`github.com`, `codeload.github.com`, `api.github.com`). Discover is not authenticated. The access token, if you log in, stays in the OS keyring. There is no analytics client in this repo.

---

## Uninstall

Log out in Settings first, so the keyring entries are deleted. Then:

```bash
rm -rf /Applications/Peti.app
rm -rf "$HOME/Library/Application Support/app.peti"
```

That does not delete the repository folders you already downloaded.

If the app is already gone and the login remains, this deletes the keyring items without printing them:

```bash
security delete-generic-password -s app.peti -a github-token
security delete-generic-password -s app.peti -a github-login
```

---

## Troubleshooting

| What you see | What to do |
|---|---|
| Paste and Discover do nothing in a browser tab | You started Vite alone. Quit it and run `npm run tauri dev`. |
| Port 1420 is in use | Vite refuses another port. Free 1420. |
| “Damaged” / cannot verify | `xattr -cr /Applications/Peti.app`, then open again. |
| `Masuk GitHub belum disiapkan.` | The client id is empty or not 1–128 of letters, digits, `.`, `_`, `-`. |
| `Masuk dibatalkan.` | GitHub refused the grant. Start login again. |
| `Kode masuk kedaluwarsa. Coba lagi.` | The device code expired. Start login again. |
| `Batas GitHub tercapai, coba lagi nanti` | GitHub rate limit. Wait, then retry. Search does not use your token. |
| `Pencarian tidak bisa dijalankan.` | Search failed for another reason. Nothing was saved. |
| `Tautan tidak valid, …` | Not a `github.com` / `www.github.com` owner/repo link. SSH is rejected. |
| `Repositori tidak tersedia` | The zip could not be fetched. |
| `Folder tidak bisa dipakai, pilih yang lain` | The destination is not a writable directory, or the open path is missing. |
| `slot unduhan penuh` | Three downloads are already running (or whatever the saved limit is, 1–8). |
| `ulang hanya untuk yang gagal` | Retry is only for a failed row. |
| `lanjut hanya untuk yang dijeda` | Resume is only for a paused row. |
| A finished folder vanished from Library | The directory was removed, or it is a symlink. Restore the same path and it shows again. The history row was kept. |
| A row failed with `ditutup saat berjalan` | The app quit while that download was running. Retry it. A paused row stays paused. |

---

## Build from source

```bash
git clone https://github.com/BadryansahBangsawan/peti.git
cd peti
npm install
npm run tauri dev
```

`npm run dev` only starts the web view. Downloads and search need `npm run tauri dev`.

The disk image, from the same directory:

```bash
npm run tauri build
```

On this Mac that command wrote:

- `src-tauri/target/release/bundle/macos/Peti.app`
- `src-tauri/target/release/bundle/dmg/Peti_0.1.0_aarch64.dmg`

The app binary is linker-signed ad-hoc (`Signature=adhoc`, no Team ID). It is not notarized. Do not commit `src-tauri/target/` or `dist/`.

Pushing a tag `v*` (for example `v0.1.0`) runs `.github/workflows/release.yml`, which builds that disk image and uploads `Peti.app.zip` plus the `.dmg`.

Checks used while developing:

```bash
npm test
npx tsc --noEmit
cargo test --manifest-path src-tauri/Cargo.toml
```

`.github/workflows/ci.yml` runs `npm test` and `cargo test` on pushes to `main` and on pull requests. That workflow has not run on GitHub yet.

Rust minimum in `src-tauri/Cargo.toml` is 1.90 (edition 2024). This machine used `rustc 1.99.0-nightly`. Node.js has no `engines` field. This machine used Node.js v26.9.0. CI uses Node.js 22. **Needs confirmation:** the oldest Node.js that works.

Longer notes, in Indonesian: [installation](docs/installation.md), [configuration](docs/configuration.md), [usage](docs/usage.md), [development](docs/development.md), [architecture](docs/architecture.md), [commands](docs/api.md), [distribution build](docs/deployment.md), [troubleshooting](docs/troubleshooting.md). Contribution notes: [CONTRIBUTING.md](CONTRIBUTING.md).

---

## FAQ

**Does Peti clone the repository?**  
No. It downloads `https://github.com/{owner}/{repo}/archive/{ref}.zip` and extracts that snapshot. There is no `.git` directory.

**Do I need a GitHub account?**  
No, for a public download or for Discover. You need an account only to fork.

**Where did the files go?**  
In the folder you picked, under `{owner}-{repo}` (or `-2`, `-3`, …). Library opens that folder.

**Why is there no Homebrew command?**  
There is no published zip to point a cask at yet.

**Why does macOS say the app is damaged?**  
The release is ad-hoc signed. It is not notarized. Run `xattr -cr /Applications/Peti.app`.

---

<div align="center">

There is no `LICENSE` file. Permission to use, modify, and distribute this project has not been set.

</div>
