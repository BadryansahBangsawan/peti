---
name: "Peti"
description: "Repository first, clear status, deliberate actions."
colors:
  peti-bg: "#FFFFFF"
  peti-side: "#F6F7F8"
  peti-panel: "#F8F9FA"
  peti-text: "#202329"
  peti-muted: "#646B75"
  peti-line: "#E7E9ED"
  peti-select: "#E8EBEF"
  peti-ink: "#25282D"
  peti-inverse: "#FFFFFF"
  peti-green: "#247354"
  peti-green-bg: "#EAF4EE"
  peti-red: "#AC4545"
  peti-red-bg: "#FFF1F0"
  peti-bg-dark: "#161719"
  peti-side-dark: "#111214"
  peti-panel-dark: "#1E2023"
  peti-text-dark: "#F2F3F5"
  peti-muted-dark: "#A4ABB5"
  peti-line-dark: "#303338"
  peti-select-dark: "#2A2D32"
  peti-ink-dark: "#F1F3F5"
  peti-inverse-dark: "#17191C"
  peti-green-dark: "#7ECBA5"
  peti-green-bg-dark: "#20392D"
  peti-red-dark: "#F0A0A0"
  peti-red-bg-dark: "#3E2729"
typography:
  headline:
    fontFamily: "Geist"
    fontSize: "30px"
    fontWeight: 600
    letterSpacing: "-0.8px"
  title:
    fontFamily: "Geist"
    fontSize: "18px"
    fontWeight: 600
  body:
    fontFamily: "Geist"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.6
  body-emphasis:
    fontFamily: "Geist"
    fontSize: "16px"
    fontWeight: 500
  navigation:
    fontFamily: "Geist"
    fontSize: "14px"
    fontWeight: 600
  label:
    fontFamily: "Geist"
    fontSize: "13px"
    fontWeight: 500
  metadata:
    fontFamily: "Geist"
    fontSize: "12px"
    fontWeight: 500
  caption:
    fontFamily: "Geist"
    fontSize: "11px"
    fontWeight: 400
  mono:
    fontFamily: "Geist Mono"
    fontSize: "15px"
    fontWeight: 400
rounded:
  progress: "2px"
  chip: "5px"
  control: "8px"
  paste: "9px"
  state: "10px"
  panel: "12px"
spacing:
  icon: "8px"
  compact: "12px"
  control-inline: "14px"
  control-group: "16px"
  section: "24px"
  compact-page: "28px"
  desktop-page: "44px"
components:
  button-primary:
    backgroundColor: "{colors.peti-ink}"
    textColor: "{colors.peti-inverse}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "10px 14px"
  button-secondary:
    backgroundColor: "{colors.peti-bg}"
    textColor: "{colors.peti-text}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "10px 14px"
  button-focus:
    backgroundColor: "{colors.peti-bg}"
    textColor: "{colors.peti-text}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "10px 14px"
  button-disabled:
    backgroundColor: "{colors.peti-ink}"
    textColor: "{colors.peti-inverse}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "10px 14px"
  search-field:
    backgroundColor: "{colors.peti-bg}"
    textColor: "{colors.peti-muted}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    padding: "0px 14px"
    height: "44px"
  navigation-selected:
    backgroundColor: "{colors.peti-select}"
    textColor: "{colors.peti-text}"
    typography: "{typography.navigation}"
    rounded: "{rounded.control}"
    padding: "0px 12px"
    height: "42px"
  chip-transfer:
    backgroundColor: "{colors.peti-green-bg}"
    textColor: "{colors.peti-green}"
    typography: "{typography.metadata}"
    rounded: "{rounded.chip}"
    padding: "4px 8px"
  panel:
    backgroundColor: "{colors.peti-panel}"
    textColor: "{colors.peti-text}"
    rounded: "{rounded.panel}"
    padding: "24px"
---

# Design System: Peti

## Overview

**Creative North Star: "Repository first"**

Peti is a quiet desktop workspace for selecting repositories and following transfers. Cool neutral surfaces, graphite actions and the original Slot symbol give the interface a precise, restrained character. Repository identity, current state and the next action stay easy to scan.

This record is extracted from the built editable canvas, its named variables and the interface-system export, rather than from running application styles. Light is the provisional primary presentation; a dark mode is also built. The direction is documented as authored, without implying a theme choice or visual approval from the user.

**Key Characteristics:**

- Solid cool neutrals and flat surfaces.
- Geist for interface text; Geist Mono for URLs and device codes.
- Repository rows own progress, status and actions.
- Text and restrained semantic color explain outcomes.

Source of truth: [editable canvas](../untitled.pen), interface-system node `jzvbx`, and `GetVariables()` read on 15 October 2026. See [BRIEF.md](BRIEF.md) for the direction contract and reference provenance, [HANDOFF.md](HANDOFF.md) for the screen map and implementation boundary, and [QA.md](QA.md) for scoped verification. Frontmatter primitives are normative; the sidecar holds extension metadata and illustrative previews.

## Colors

The palette is solid, cool and low in saturation. Frontmatter keys retain the canvas variable names. Unsuffixed keys hold light values; `-dark` keys hold the corresponding `mode: dark` values. The canvas uses one semantic variable per pair, not separate light/dark variable names.

### Primary

Action graphite (`peti-ink`) and action contrast (`peti-inverse`) form primary controls. The pair reverses in dark mode so the action remains distinct without introducing a brand accent.

### Semantic status

Transfer green (`peti-green`, `peti-green-bg`) identifies active/successful transfer information. Recovery red (`peti-red`, `peti-red-bg`) identifies failure and recovery information. These are functional status colors, not extra action colors.

### Neutral

Canvas (`peti-bg`) is the main content surface. Cool sidebar (`peti-side`) separates navigation; quiet surface (`peti-panel`) groups summaries and supporting information. Reading ink (`peti-text`) carries identity and primary labels; secondary ink (`peti-muted`) carries metadata. Boundary (`peti-line`) divides rows and controls, while selected surface (`peti-select`) identifies the active destination.

**The Status Rule.** Use green and red to explain state alongside text; keep the primary action neutral.

Measured text-pair contrast is recorded in QA.md. Disabled opacity is an illustrative state exception. Tonal ramps in the sidecar are generated preview metadata, not additional authored palette choices.

## Typography

Geist carries headings, repository identity, body text, navigation and controls. Geist Mono appears only in URL/device-code specimens. The frontmatter records observed roles; it does not invent a geometric type scale or universal line height. The body line height applies to wrapped descriptive copy; many single-line specimens use the font default.

Page headings use `headline`, state headings use `title`, and normal content spans the `body` and `body-emphasis` roles. Controls use `label`; secondary information uses `metadata` or `caption`. The system-board title is a display specimen rather than an additional application heading token.

**The Mono Rule.** Reserve Geist Mono for repository URLs and device authorization codes.

## Layout

Desktop examples are 1440 × 1000, with a 220 px sidebar and 66 px toolbar. Page content has 44 px horizontal padding, 34 px top padding, 26 px bottom padding and a dominant 24 px section gap. The toolbar has 40 px horizontal padding. Repository lists use shared rows and dividers instead of placing every row inside a card.

The 1100 × 800 compact example uses a 72 px icon rail, 56 px toolbar, 28 px horizontal page padding, 24 px top padding, 20 px bottom padding and 12 px section gaps. Sidebar destination details are hidden in this example; the active page and row actions remain visible. These are two authored viewport specimens, not a tested breakpoint policy or minimum window size. No mobile layout is specified.

Spacing keys cover recurring distances rather than every one-off measurement. The system board has its own presentation layout and is not an app-page template.

## Elevation & Depth

Surfaces are flat. Tonal shifts distinguish sidebar, summary panel, selected navigation and main content; fine borders divide related information. Focus uses an outer stroke rather than shadow. There are no authored drop shadows or gradients.

**The Flat Surface Rule.** Separate surfaces with tone, spacing and quiet borders; use no shadows or gradients.

## Shapes

Controls have gently curved corners (`rounded.control`); chips are tighter (`rounded.chip`) and summary/action panels are broader (`rounded.panel`). Preserve the observed exceptions: the quick-paste composer uses `rounded.paste`, state cards and the sidebar destination block use `rounded.state`, and progress tracks use `rounded.progress`. Keep the exact original Slot path geometry when sizing the brand mark.

## Components

### Buttons

Primary and secondary controls share label type, compact padding and an icon/label gap (`spacing.icon`). Primary uses action graphite/inverse; secondary uses canvas/reading ink with a single-pixel boundary stroke. The keyboard-focus specimen has an outer 2 px reading-ink stroke. The disabled primary specimen has opacity 0.4. Four button specimens are reusable, with four primary/secondary instances placed in the authored screens. Hover and pressed colors are not separately authored; do not claim an unobserved treatment as a canvas token.

The system board calls for quiet transitions of 100–150 ms and reduced-motion support. These are implementation intent; animation and keyboard traversal have not been runtime-tested.

### Inputs / Fields

The Discover search field is 44 px high, with control corners, a single-pixel boundary, 14 px horizontal padding, a 17 px search icon and 13 px placeholder copy. The quick-paste composer is a distinct bordered row with paste corners and 13 px vertical / 16 px horizontal padding. Validation separates valid and invalid repository entries and retains a clear recovery action.

### Navigation

Selected desktop navigation uses the selected surface, reading ink, a 42 px row and 12 px horizontal inset. It combines an 18 px icon, a 14 px semibold label and an optional count. Default destinations use secondary text. The compact variant presents icons in the narrower rail; accessible names, tooltips and keyboard behavior require implementation verification.

### Chips

Status chips have chip corners and compact 4 px vertical / 8 px horizontal padding. Transfer and failure chips pair their semantic text/surface colors with an explicit label. Neutral branch, count and shortcut chips use secondary ink on quiet surfaces. Chips do not replace primary actions.

### Cards / Containers

Summary and action panels use panel corners and quiet surfaces, commonly with 24 px padding; the compact transfer summary reduces its inset to 18 px. State cards use state corners, a single-pixel boundary and 24 px padding. Keep explanatory content together without turning every repository into a floating card.

### Repository progress and outcomes

A repository row groups identity, branch, transfer details and local controls. Known-size tracks are 4 px high, with fill proportional to received / total bytes; unknown-size rows omit a guessed percentage and ETA. Pause, cancel, retry and resume states must follow real transfer state. Resume that cannot use HTTP Range explains that transfer starts again.

The 16 reusable interaction/outcome states cover: Downloads empty; Discover no results; loading; connection failure; missing local folder; GitHub limit; expired device code; GitHub disconnected; success feedback; download cancellation; fork success; fork plus download completion; restart on resume; Library empty; repository unavailable; and destination permission failure. The last six appear on the outcomes board. Library completion points to the actual folder; fork results identify the created owner/repository. Pending authorization does not imply a connected account.

Sidecar component previews translate these visual specimens into isolated HTML/CSS examples. They are documentation renderers, not application implementation or a functional prototype.

## Do's and Don'ts

### Do:

- **Do** keep the original Peti Slot geometry and product name.
- **Do** reuse the canvas light/dark variables by semantic role.
- **Do** preserve repository identity, status text and a clear next action.
- **Do** derive known progress from bytes; show bytes and speed when size is unknown.
- **Do** make fork actions explicit and identify the fork and actual local folder in results.
- **Do** retain visible keyboard focus and respect reduced motion during implementation.

### Don't:

- **Don't** add decorative shadows, gradients or replacement logo geometry.
- **Don't** use monospace as the general interface typeface.
- **Don't** communicate error, completion or selection with color alone.
- **Don't** infer a responsive breakpoint or minimum window size from two canvas examples.
- **Don't** treat canvas states or preview snippets as working controls or tested runtime behavior.
