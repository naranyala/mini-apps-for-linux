# Technical TODOs

This document is the implementation roadmap for growing Mini Apps for Linux into a polished collection of small desktop tools. It distinguishes the current application from proposed work: checklist items below are not claims that a feature is already implemented.

## Product direction

- [ ] Keep Mini Apps as one native Linux application with a searchable launcher and integrated tool panels.
- [ ] Build new concepts as **mockups first**: interactive UI, seeded sample data, and state held in memory only.
- [ ] Make mockup boundaries visible in the UI and documentation. Do not imply that simulated system metrics, networks, devices, files, or repositories are live data.
- [ ] Do not add system-changing actions to mockups. No network configuration, audio-device changes, power controls, package installation, filesystem scanning, Git access, or log-file access.
- [ ] Preserve the existing four tools and their documented behaviors while adding mockups.
- [ ] Prefer small, discoverable tools with a clear primary interaction over broad settings dashboards.

## Phase 1 — Establish a maintainable foundation

- [ ] Split `src/main.rs` into modules while keeping the current application behavior:
  - [ ] `app` — launcher, navigation, and shared app state.
  - [ ] `theme` — palette and egui theme setup.
  - [ ] `tools` — tool catalog and tool IDs.
  - [ ] `tools/shortcut_maker.rs` — existing Desktop Shortcut Maker.
  - [ ] `tools/clipboard.rs` — existing Clipboard Inspector.
  - [ ] `tools/screenshot.rs` — existing Screenshot Tool and portal capture support.
  - [ ] `tools/color_picker.rs` — existing Color Picker and sampling canvas.
  - [ ] `ui` — shared cards, form controls, status pills, and headers.
- [ ] Keep app-specific state local to each tool; avoid adding unrelated tool state to one global struct.
- [ ] Define a consistent convention for tool metadata: stable ID, display name, summary, search keywords, and mockup/functional status.
- [ ] Add reusable UI patterns for empty states, sample-data labels, status feedback, and responsive layouts.
- [ ] Preserve existing unit tests during the module split and run `cargo test --locked` before and after the refactor.
- [ ] Check formatting with `cargo fmt --check` and address compiler warnings introduced by the refactor.

## Phase 2 — Add the first mockup batch

Implement each item as a self-contained tool panel using local state and deterministic sample data. Buttons and controls should visibly respond, but must not perform external system actions.

### Quick Notes

- [ ] Add a list/detail layout with seeded example notes, title, body, and tags.
- [ ] Implement local search, tag filtering, pin/unpin, and note selection.
- [ ] Provide a preview mode for a small, documented subset of Markdown-like formatting.
- [ ] Keep edits in memory only; label the notes as a demo and do not write user files.
- [ ] Add tests for search/filter behavior and any pure formatting helpers.

### System Monitor

- [ ] Create dashboard cards for CPU, memory, disk, and network using clearly labeled sample values.
- [ ] Add charts driven by deterministic or bounded mock data; label them as simulated.
- [ ] Include a table of fictional processes and basic sorting/filtering interactions.
- [ ] Do not read `/proc`, query real services, or imply that values describe the host machine.
- [ ] Test sample-data bounds and sorting/filtering logic.

### Audio Mixer

- [ ] Add a mock output-device selector and fictional per-app volume rows.
- [ ] Implement local slider, mute, and master-volume interactions.
- [ ] Add a visual level meter using simulated values and mark it as a demo.
- [ ] Do not connect to PipeWire, PulseAudio, ALSA, or modify actual audio settings.
- [ ] Test volume clamping and mute/unmute state transitions.

### Pomodoro Timer

- [ ] Add focus, short-break, and long-break modes with configurable display presets.
- [ ] Implement start, pause, reset, and mode selection with a visible countdown.
- [ ] Keep completed-session history in memory and provide a seeded example or clear empty state.
- [ ] Avoid background notifications or persistent tracking in the initial mockup.
- [ ] Test timer state transitions and duration calculations without relying on wall-clock sleeps.

## Phase 3 — Grow the collection

- [ ] Add a **Unit Converter** with category tabs, input swapping, favorites, and recent conversions; cover conversion formulas with tests.
- [ ] Add **Color Palette Studio** for sample palettes, swatches, and contrast comparisons; keep it distinct from the existing screen Color Picker.
- [ ] Add a **Downloads Organizer** using fictional files, type groups, filters, and a mock preview; do not enumerate the user's Downloads directory.
- [ ] Add a **Git Dashboard** with fictional repositories and commit histories; do not invoke Git or access repository credentials.
- [ ] Add a **Log Viewer** over bundled/sample log lines with severity filters; do not read host logs.
- [ ] Add further concepts from README only when they have a distinct user need, a mock-data plan, and a small set of testable interactions.

## Launcher and interaction quality

- [ ] Add a visible distinction between functional tools and mockups in the launcher and tool screens.
- [ ] Ensure search matches tool names, descriptions, and keywords, including newly added tools.
- [ ] Verify keyboard navigation, Enter-to-open, Escape/back behavior, and focus restoration.
- [ ] Check the tool grid and each panel at the minimum window size and at wider sizes.
- [ ] Ensure long text, empty results, and sample-data tables remain usable in scroll areas.
- [ ] Provide consistent status feedback and avoid silent no-op controls.
- [ ] Keep the application responsive; move any future slow operations off the UI thread, but do not add background system polling for mockups.

## Safety, privacy, and data handling

- [ ] Document which existing tools access clipboard contents, invoke desktop portals, create `.desktop` files, or call `xdg-open`.
- [ ] Keep each mockup's sample state in memory unless persistent storage is separately designed and explicitly disclosed.
- [ ] Never present fictional values as live system data.
- [ ] Avoid collecting, transmitting, or persisting personal data in mockups.
- [ ] For any future real system integration, require an explicit design review, clear user initiation, error handling, and documentation of permissions and side effects before implementation.

## Testing and release readiness

- [ ] Run `cargo fmt --check` and `cargo test --locked` for every change.
- [ ] Add unit tests for non-UI behavior, including filtering, calculations, state transitions, and formatting.
- [ ] Manually verify all four existing tools on a supported Linux desktop session after architectural changes.
- [ ] Smoke-test new mockups without network access and verify that they operate only on sample/local in-memory data.
- [ ] Update README.md when tool status, dependencies, requirements, build instructions, or mockup availability changes.
- [ ] Keep requirements accurate: portal and clipboard dependencies should be described only for tools that use them.
- [ ] Before packaging or distribution, resolve the project's license status; no license is currently declared in the repository.

## Definition of done for a new mockup

A mockup is ready to merge when all of the following are true:

1. Its launcher entry and panel use shared navigation and visual conventions.
2. Its sample/demo nature is explicit wherever a user could mistake data for live system state.
3. Its interactions are functional within the app and do not perform undisclosed external actions.
4. It has a useful empty state, handles narrow window sizes, and provides feedback for user actions.
5. Pure behavior has tests, and `cargo fmt --check` plus `cargo test --locked` pass.
6. README.md accurately lists the tool as a mockup rather than a fully integrated system utility.
