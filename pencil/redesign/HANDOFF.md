# Peti — design handoff

The editable source is [untitled.pen](../untitled.pen). Open it with pen.dev; the file is encrypted and should not be parsed or edited as raw JSON. [PDF preview](../exports/export.pdf).

## Canvas map

| Surface | Node | Export |
|---|---|---|
| Interface system | jzvbx | [PNG](../exports/jzvbx.png) |
| Home | dNqBM | [PNG](../exports/dNqBM.png) |
| Downloads | bi8Au | [PNG](../exports/bi8Au.png) |
| Discover + batch selection | BBYTP | [PNG](../exports/BBYTP.png) |
| Library | GHft8 | [PNG](../exports/GHft8.png) |
| Settings | u7oV2P | [PNG](../exports/u7oV2P.png) |
| Add repositories + validation | gyYw1 | [PNG](../exports/gyYw1.png) |
| Repository details + action choice | nePXF | [PNG](../exports/nePXF.png) |
| GitHub device authorization | hXGlb | [PNG](../exports/hXGlb.png) |
| Downloads dark | yfmwD | [PNG](../exports/yfmwD.png) |
| Downloads compact | I43eG | [PNG](../exports/I43eG.png) |
| 10 interaction states | L0SL9 | [PNG](../exports/L0SL9.png) |
| 6 download/fork outcomes | zHYy1 | [PNG](../exports/zHYy1.png) |

### Added continuation pages

| Surface | Node | Export |
|---|---|---|
| Download detail, stages and destination | D3dVe | [PNG](../exports/D3dVe.png) |
| Downloads completed tab | u6DMbQ | [PNG](../exports/u6DMbQ.png) |
| Downloads failed tab and recovery | hfcso | [PNG](../exports/hfcso.png) |
| Filtered search with language menu | xwyzo | [PNG](../exports/xwyzo.png) |
| Home first use, without account | M84aN | [PNG](../exports/M84aN.png) |
| Library empty, without account | g4UYA | [PNG](../exports/g4UYA.png) |
| Settings without GitHub account | NsvLt | [PNG](../exports/NsvLt.png) |
| Fork created and download queued | l7d71C | [PNG](../exports/l7d71C.png) |
| Home dark | s4kpSi | [PNG](../exports/s4kpSi.png) |
| Discover dark | zKMk8 | [PNG](../exports/zKMk8.png) |
| Library dark | P7Z8yY | [PNG](../exports/P7Z8yY.png) |
| Settings dark | gPMnS | [PNG](../exports/gPMnS.png) |

The canvas now contains 22 app screens/variants and three reference boards. The new pages start at canvas y=3980: detail/completed/failed/filtered search in the first row, first-use/Library empty/guest Settings/fork result at y=5100, and the four dark pages at y=6220. The prior screens and boards retain their positions. The PDF includes all 25 frames.

Main screens are 1440 × 1000; compact is 1100 × 800. The system board sits above the screens. Each root is clipped and labelled; reusable controls/states remain editable. Theme variables support light and dark.

## Interaction intent

1. Home paste → Add repo validation and destination → Downloads → Library/open folder.
2. Discover search/filter → details or checkbox selection → shared queue → Library.
3. Settings → GitHub device flow → explicit action choice → fork identity and/or local folder result.

The continuation adds full-page examples for transfer inspection, completion and recovery. The fork result deliberately shows a created fork with a waiting download, rather than claiming the local folder is already ready. The first-use and empty-Library screens show guest identity and zero unfinished downloads; guest Settings keeps the existing queue intact and selects the system theme. Dark Settings selects the dark theme. The open language menu sits above result rows; close it before selecting a covered result.

Actions, focus/disabled specimens and result states are visual specifications. The canvas is not a functional prototype, and no app controls were wired or deployed.

## Implementation boundary

The existing frontend currently renders queue status/error rows. The design adds the detailed progress and controls from the product plan. Inspect the Tauri command surface before implementing: received/total bytes, speed, ETA and pause/resume/cancel/retry must be exposed to the UI and derived from real transfer state.

Home summary, paste composer, folder/concurrency controls, Library search/sort and branch chooser are proposed interface work; their final data bindings require implementation. Existing Discover search, history/open-folder, theme and GitHub code provide starting integration points. Preserve the existing tests and source behavior during that future work.

Unknown size means no guessed percentage or ETA. Resuming without HTTP Range must clearly say it restarts from the beginning. Library includes completed items only. Fork success identifies the created owner/repo; fork+download identifies both the fork and actual extracted folder.

## Runtime limit

The current application could not be launched in this session because binding a local preview port was denied (`EPERM`). Review used source/specification inspection, canvas screenshots, layout bounds and exported design review. Real app behavior, native folder picker, keyboard traversal, motion, and narrower/minimum-size windows still require implementation testing.
