# Technical TODOs

This document is the execution plan for growing Mini Apps for Linux into a richer collection of polished desktop mini-apps while keeping the current application honest: real features remain real, and mockups remain simulated.

This repo already contains a working Rust desktop app with four real tools:

- Desktop Shortcut Maker
- Clipboard Inspector
- Screenshot Tool
- Color Picker

The roadmap below is intentionally broader: it covers how to turn the app from a small launcher into a curated mini-apps suite without blurring the line between real capabilities and UI-only concepts.

## 1. Product goals

- [ ] Keep Mini Apps as a single native Linux desktop app with a unified launcher and shared visual language.
- [ ] Make the app feel like a collection of small utilities, not a fragmented toolkit with unrelated screens.
- [ ] Add future mini-apps intentionally as mockups first, with clear labeling and sample data.
- [ ] Preserve current functionality and avoid regressions in the four existing tools.
- [ ] Make the app feel lightweight, dense with useful information, and unmistakably Linux-first.
- [ ] Keep every new concept small and focused: one primary interaction, one primary data model, one primary screen.

## 2. Non-goals and guardrails

This is where we protect the product from becoming misleading or invasive.

- [ ] No real system mutation in mockup screens.
- [ ] No reading real logs, processes, network config, audio devices, or package metadata in mockups.
- [ ] No writing user files or persisting real data in mockup tools without explicit future design review.
- [ ] No claiming a simulated value is live system state.
- [ ] No hidden background tasks for mockups that silently access privileged resources.
- [ ] If a tool needs system integration later, it must be introduced as a real feature with explicit consent and documented permissions.

## 3. Design system and UI standards

Before adding more mini-apps, define the shared visual and interaction language.

- [ ] Define a visual token set:
  - [ ] app background color, panel color, card color, border color
  - [ ] accent colors for primary actions and selected states
  - [ ] success, warning, and error states
  - [ ] text colors for heading, body, muted, and code-like labels
- [ ] Define reusable UI primitives:
  - [ ] header block
  - [ ] card frame
  - [ ] status pill
  - [ ] segmented control
  - [ ] filter/search field
  - [ ] inline action row
  - [ ] empty state component
- [ ] Establish a standard for mockup labeling:
  - [ ] each mockup should say “demo”, “sample data”, or “simulated” in the UI where relevant
  - [ ] mockup-only screens should avoid using real-sounding labels like “connected”, “active”, “real disk usage”, or “live processes”
- [ ] Define layout rules:
  - [ ] narrow windows should stack vertically
  - [ ] wide windows should use 2-column or 3-column layouts when the content benefits from it
  - [ ] long lists should live inside scroll areas with clear visual affordances
- [ ] Keep all icons and labels consistent with the existing Catppuccin-inspired dark theme.

## 4. Architecture and repo structure plan

The current codebase is a single-file app (`src/main.rs`). More features will become difficult to maintain unless we split responsibilities.

### Planned module layout

- [ ] `src/main.rs`
  - [ ] keep only bootstrapping and app setup
  - [ ] optionally remove most feature logic from the top-level file
- [ ] `src/app.rs`
  - [ ] root app state and app lifecycle
  - [ ] launcher state and tool selection
- [ ] `src/theme.rs`
  - [ ] dark theme palette
  - [ ] widget styling
  - [ ] common colors and spacing constants
- [ ] `src/tools/mod.rs`
  - [ ] tool catalog definitions
  - [ ] tool IDs and metadata
- [ ] `src/tools/shortcut_maker.rs`
  - [ ] current real feature logic
- [ ] `src/tools/clipboard.rs`
  - [ ] current real feature logic
- [ ] `src/tools/screenshot.rs`
  - [ ] current real feature logic
- [ ] `src/tools/color_picker.rs`
  - [ ] current real feature logic
- [ ] `src/tools/mockups/`
  - [ ] `notes.rs`
  - [ ] `system_monitor.rs`
  - [ ] `audio_mixer.rs`
  - [ ] `focus_timer.rs`
  - [ ] `unit_converter.rs`
  - [ ] `downloads.rs`
  - [ ] `palette_studio.rs`
  - [ ] other future mockup modules
- [ ] `src/ui.rs`
  - [ ] shared frame, header, button, list, card components
- [ ] `src/data.rs`
  - [ ] sample data stores for mockup screens

### Refactor steps

- [ ] Create module skeleton without changing behavior.
- [ ] Move tool-specific state and UI code behind tool modules.
- [ ] Keep public app behavior identical during the extraction.
- [ ] Add a stable tool metadata format for both real and mockup apps.
- [ ] Ensure each tool module owns its state and UI rendering.
- [ ] Avoid making global state a dumping ground for unrelated tool data.

## 5. Milestone roadmap

### Milestone 0 — Foundation cleanup and stability

- [ ] Split the app into modules while preserving current behavior.
- [ ] Add shared UI helpers to reduce repeated code.
- [ ] Ensure all four real tools still work after refactoring.
- [ ] Add targeted tests for the helper code and the Desktop Shortcut Maker logic.
- [ ] Confirm formatting and tests pass on the refactored code.

### Milestone 1 — First mockup batch

Implement the first four mockups as UI-only, local-state concepts.

- [ ] Quick Notes
- [ ] System Monitor
- [ ] Audio Mixer
- [ ] Focus Timer

Delivery bar:

- [ ] All four feel like a unified collection
- [ ] Sample data is clearly identified as simulated
- [ ] No external system calls are made
- [ ] They are fully interactive within the app

### Milestone 2 — Expand the suite

Add second-wave mockups that broaden the utility set without expanding scope too much.

- [ ] Unit Converter
- [ ] Color Palette Studio
- [ ] Downloads Organizer
- [ ] Git Dashboard or Developer Overview (demo-only)
- [ ] Log Viewer (demo-only)

Delivery bar:

- [ ] Each tool has a clear purpose and primary interaction
- [ ] All mockup screens share the same navigation style
- [ ] Each screen’s demo nature is noticeable without being obnoxious

### Milestone 3 — Polish and release-readiness

- [ ] Improve empty states and keyboard accessibility.
- [ ] Review all interactions for clarity.
- [ ] Standardize naming, labels, and helper text.
- [ ] Verify the launcher search works with all new tools.
- [ ] Review README and TODOs for correctness after each milestone.

## 6. Tool-by-tool backlog

### 6.1 Quick Notes

Goals:

- [ ] Create a compact, easy-to-scan note-taking tool.
- [ ] Use a clear list/detail layout.
- [ ] Keep the app local and fully in-memory.

Implementation tasks:

- [ ] Add a note list with title, tag, and preview text.
- [ ] Support note selection and contextual detail panel.
- [ ] Add search and tag filters.
- [ ] Add pin/unpin behavior.
- [ ] Add a small preview mode for Markdown-like formatting.
- [ ] Add sample notes and a clear empty-state layout.
- [ ] Keep edits ephemeral and clearly marked as demo-only.
- [ ] Add pure helper tests for filtering and content preview logic.

Acceptance criteria:

- [ ] It feels like a lightweight notes app, not an editor.
- [ ] Search and filter work reliably.
- [ ] Notes are in-memory only.
- [ ] The UI clearly states that it is a mockup.

### 6.2 System Monitor

Goals:

- [ ] Show a glanceable dashboard with fictional values.
- [ ] Feel like a small utility for checking system health at a glance.

Implementation tasks:

- [ ] Add metric cards for CPU, memory, storage, and network.
- [ ] Add simulated charts with predictable values.
- [ ] Add a process table or activity list with fake entries.
- [ ] Add labels to indicate “simulated data only”.
- [ ] Keep it visually compact and readable.
- [ ] Add sorting or filtering for process entries.
- [ ] Add a warning/healthy styling pattern for metrics.
- [ ] Unit-test sorting and metric-normalization logic.

Acceptance criteria:

- [ ] No code reads real host metrics.
- [ ] Values are clearly marked as intentionally fictional.
- [ ] The layout remains readable at small sizes.

### 6.3 Audio Mixer

Goals:

- [ ] Show a polished, compact mixer concept.
- [ ] Offer per-app style controls.

Implementation tasks:

- [ ] Add master volume and per-app volume sections.
- [ ] Add mute toggles for each app row.
- [ ] Add mock device selection.
- [ ] Add a visual output meter using static or pseudo-animated data.
- [ ] Add a “Demo device” label and an explicit note that the tool does not connect to audio services.
- [ ] Ensure slider changes are local-only and visible immediately.
- [ ] Add tests for slider clamp logic and mute state transitions.

Acceptance criteria:

- [ ] UI is responsive and polished.
- [ ] Logic is local and deterministic.
- [ ] No actual system sound output or configuration is touched.

### 6.4 Focus Timer

Goals:

- [ ] Provide a compact timer concept for work sessions.

Implementation tasks:

- [ ] Add focus, short-break, and long-break presets.
- [ ] Implement a visible countdown.
- [ ] Add start, pause, reset actions.
- [ ] Add completed-session history with seeded sample data.
- [ ] Add a small weekly streak chart or `today` summary.
- [ ] Keep timer behavior based on local state and not on real background OS timing.
- [ ] Add unit tests for countdown math, mode transitions, and session summary calculations.

Acceptance criteria:

- [ ] Timer logic is deterministic and testable.
- [ ] It remains a visual prototype without OS notifications or persistent tracking.

### 6.5 Unit Converter

Goals:

- [ ] Offer a quick, easy conversion workflow.

Implementation tasks:

- [ ] Add category tabs such as length, temperature, weight, volume.
- [ ] Add paired inputs and a swap button.
- [ ] Add recent conversion list.
- [ ] Add support for common favorite conversions.
- [ ] Write conversion helpers and test them separately from UI code.

### 6.6 Color Palette Studio

Goals:

- [ ] Extend the app’s creative side without duplicating Color Picker.

Implementation tasks:

- [ ] Add palette cards and swatch previews.
- [ ] Include contrast-check styling for text readability.
- [ ] Add color harmony suggestions.
- [ ] Add export/save-to-copy interactions for hex values.
- [ ] Keep it clearly as a design mockup, not a real screen-world sampler.

### 6.7 Downloads Organizer

Goals:

- [ ] Simulate file organization and sorting without reading the real filesystem.

Implementation tasks:

- [ ] Add file categories and mock metadata.
- [ ] Add search and sorting controls.
- [ ] Add a detail preview panel.
- [ ] Keep all data seeded and synthetic.

### 6.8 Git Dashboard / Developer Overview

Goals:

- [ ] Offer a lightweight summary screen for a dev-oriented mockup.

Implementation tasks:

- [ ] Add fake repo cards with statuses and commit times.
- [ ] Add branch and PR badges using demo values.
- [ ] Ensure no Git commands are executed.
- [ ] Clearly label the screen as a sample overview.

### 6.9 Log Viewer

Goals:

- [ ] Simulate a developer log viewer with style and filtering.

Implementation tasks:

- [ ] Add severity filters.
- [ ] Add a search box and sample line detail panel.
- [ ] Add severity color coding.
- [ ] Keep the entries synthetic and bundled locally.

## 7. Interaction and QA checklist

For each new entry, verify the following before merging:

- [ ] Search works across names, descriptions, and keywords.
- [ ] Back navigation works with click and keyboard interactions.
- [ ] Empty states are visible and not confusing.
- [ ] Scroll areas do not break layout.
- [ ] The app remains usable at both narrow and wide window sizes.
- [ ] Buttons are identifiable as actions and not silent no-ops.
- [ ] Sample-data labels are visible where needed.
- [ ] There are no hidden system calls or file accesses.
- [ ] The interface remains consistent with the rest of the app.

## 8. Testing strategy

### Unit tests

- [ ] Add tests for helper logic:
  - [ ] filtering and search ranking
  - [ ] timer logic and countdown math
  - [ ] unit conversion helpers
  - [ ] note preview functions
  - [ ] progress normalization and scaling
- [ ] Keep tests deterministic and avoid real-time waits.

### Manual verification

- [ ] Run the app locally and verify each tool panel loads.
- [ ] Test launcher search and selection.
- [ ] Verify that mockups never imply real system data.
- [ ] Verify the app still works with the four real tools after adding mockups.

### CI checks

- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo test --locked`.
- [ ] Confirm no warnings or errors introduced by the refactor.

## 9. Documentation updates

- [ ] Update README.md when new mockups are added.
- [ ] Update TODOs and roadmap when design or scope changes.
- [ ] Document which screens are real and which screens are mockups.
- [ ] Keep the language explicit: “interactive mockup”, “sample data”, “demo”, or “simulated”.

## 10. Release-readiness definition

A new mini-app or mockup is ready to merge when all of the following are true:

1. It fits the launcher and the app’s styling system.
2. It is clearly labeled as a mockup if it does not access real system data.
3. It is fully interactive and understandable without external scaffolding.
4. It has a clear purpose and a narrow scope.
5. It passes testing and formatting checks.
6. It has no hidden side effects or real-world system access.
7. README.md and the roadmap remain accurate.

## 11. Immediate next actions

Before adding more mockups, the top-priority work is:

- [ ] modularize the current app
- [ ] centralize theme and API patterns
- [ ] land the first mockup batch (Quick Notes, System Monitor, Audio Mixer, Focus Timer)
- [ ] validate search and layout quality across all tools
- [ ] ensure every mockup is explicitly demo-only

This keeps the project moving without turning the app into a misleading simulation of real desktop functionality.
