# Mini Apps for Linux

Mini Apps is a small Linux desktop utility launcher built as one Rust application, not a collection of separately packaged programs. It currently has four functional desktop tools, with room to grow into a broader collection of polished, self-contained mini-apps.

## Included tools

- **Desktop Shortcut Maker** — create a FreeDesktop `.desktop` launcher for a binary. Set its name and executable path, and optionally an icon, description, categories, and whether it should launch in a terminal.
- **Clipboard Inspector** — monitor and search a session-only history of copied text. The history is limited to 100 entries and can be paused, cleared, or copied again.
- **Screenshot Tool** — capture a region or the full screen through the desktop screenshot portal. View recent captures, copy a capture path, or open it with `xdg-open`.
- **Color Picker** — capture a screen region through the desktop portal, select pixels from the captured image, and copy their color as an uppercase `#RRGGBB` value.

The launcher has fuzzy search across tool names and keywords, and its tool cards reflow to fit the window. The application opens with a dark theme.

Clipboard history is held in memory for the current app session and is not saved to disk. The watcher reports initialization errors and offers a retry action; new non-empty text copied after monitoring starts is added to the history.

## More mini-app mockup ideas

These are **ideas for future mockups, not features currently implemented**. They can make the collection feel rich and varied while remaining safe to explore: use sample data, keep interactions local to the app, and avoid changing system settings or writing files.

### Productivity and personal tools

- **Quick Notes** — searchable note cards with tags, pins, and a Markdown-style preview, populated with example notes.
- **Pomodoro Timer** — focus and break modes, session presets, a task field, and a sample daily streak.
- **Habit Tracker** — weekly activity grid, goals, streak cards, and fictional history.
- **Calendar & World Clock** — month view, sample events, timezone cards, and a meeting-time comparison.
- **Unit Converter** — category tabs, paired values, a swap action, favorites, and recent conversions.
- **Recipe Cards** — sample recipes with servings, ingredient checklists, and a mock timer.

### Desktop and system dashboards

- **System Monitor** — simulated CPU, memory, disk, and network charts alongside sample process rows.
- **Wi-Fi & Network Dashboard** — mock networks, signal indicators, a fictional connection state, and sample traffic history; no network changes.
- **Audio Mixer** — mock output devices, per-app volume sliders, mute controls, and animated level meters; no audio-device control.
- **Power & Battery** — sample charge level, estimated runtime, usage chart, and power-mode cards; no power-setting changes.
- **Disk Visualizer** — a sample filesystem treemap, breadcrumbs, file-type legend, and largest-items list; no filesystem scanning.
- **Window Layout Planner** — arrange sample windows into desktop zones and preview layouts without moving real windows.

### Creative and reference tools

- **Color Palette Studio** — organize sample palettes, inspect swatches, and compare contrast. This complements the existing screen-sampling Color Picker rather than duplicating it.
- **Font Previewer** — browse bundled sample font names, compare sizes and weights, and preview sample text.
- **Typography Pairing Board** — try headline/body font pairings against a few built-in layout cards.
- **Emoji & Symbol Browser** — searchable categories, a details panel, and a mock recently-used list.
- **Keyboard Shortcut Explorer** — browse categorized example shortcuts with a key-combination visualizer.
- **Markdown Playground** — edit and preview an in-memory sample document with formatting toolbar examples.
- **QR Code Studio** — preview a code generated from example text, with styling controls that remain in the mockup.

### Developer and file-workflow demos

- **Downloads Organizer** — fictional recent downloads grouped by file type, with sorting, filters, and a mock preview panel.
- **Git Dashboard** — sample repositories, branch cards, commit timelines, and status indicators without connecting to Git.
- **Log Viewer** — searchable sample log lines, severity filters, and a detail panel; no reading system logs.
- **Package Explorer** — a curated mock package list with descriptions, categories, and update badges; no package-manager actions.

A good first mockup batch would be **Quick Notes, System Monitor, Audio Mixer, and Pomodoro Timer** because they introduce different layouts and interactions without overlapping much with the current tools. The mockup concepts above are a roadmap, not a claim that those tools already exist.

## Requirements

- Linux and a graphical desktop session.
- A current stable Rust toolchain and Cargo. The package uses the Rust 2024 edition; the repository does not declare a minimum supported Rust version.
- A working desktop screenshot portal for the Screenshot Tool and Color Picker. The portal determines where captures are saved (often `~/Pictures`).
- Clipboard access for clipboard monitoring and copying. The Screenshot Tool's **Open in Image Viewer** action also requires `xdg-open`.

The repository does not document distro-specific system package prerequisites. Native GUI libraries required by the Rust dependencies may vary by distribution and desktop environment.

## Build and run

From the repository root:

```sh
cargo run --locked
```

To build an optimized executable:

```sh
cargo build --release --locked
```

The resulting binary is `target/release/create-linux-desktop-shortcut`. Run it directly to launch the same application.

Run the test suite with:

```sh
cargo test --locked
```

## Installation and removal

The repository does not include a system installer, release package, or application uninstall script. `cargo run` runs the app from the source checkout; `cargo build --release` creates a local binary in `target/release/`.

Desktop Shortcut Maker is a feature within the app, not an installer for Mini Apps. It creates a launcher entry for the binary you select at `~/.local/share/applications/<name>.desktop` (the app creates that directory if needed and sets the entry's permissions to `0755`).

## Repository layout

| Path | Contents |
| --- | --- |
| `Cargo.toml` | Package metadata and Rust dependencies |
| `Cargo.lock` | Locked dependency versions |
| `src/main.rs` | Application, tools, and unit tests |
| `inspiration/` | A Git submodule entry; this checkout has no submodule contents, and the repository does not include a `.gitmodules` mapping for it |

## Contributing

Bug reports and focused improvements are welcome. Before opening a pull request, run the test suite:

```sh
cargo test --locked
```

Include a concise description of the change and relevant test results. No project-specific contribution guide is currently included.

## License

No license file or license declaration in `Cargo.toml` is present in this repository. Clarify licensing with the maintainers before reusing or distributing the code.
