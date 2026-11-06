# Mini Apps for Linux

Mini Apps is a small Linux desktop utility launcher. It is currently a single Rust application with four built-in tools; it is not a collection of separately packaged programs.

## Included tools

- **Desktop Shortcut Maker** — create a FreeDesktop `.desktop` launcher for a binary. Set its name and executable path, and optionally an icon, description, categories, and whether it should launch in a terminal. The generated entry is written to `~/.local/share/applications/`.
- **Clipboard Inspector** — monitor and search a session-only history of copied text. The history is limited to 100 entries and can be paused, cleared, or copied again.
- **Screenshot Tool** — capture a region or the full screen through the desktop screenshot portal. View recent captures, copy a capture path, or open it with `xdg-open`.
- **Color Picker** — capture a screen region through the desktop portal, select pixels from the captured image, and copy their color as an uppercase `#RRGGBB` value.

The launcher has fuzzy search across tool names and keywords, and its tool cards reflow to fit the window. The application opens with a dark theme.

Clipboard history is held in memory for the current app session and is not saved to disk. The watcher reports initialization errors and offers a retry action; new non-empty text copied after monitoring starts is added to history. Clipboard write failures are shown in the tool's status message. Screenshot and color-sampling requests use the desktop portal, so the desktop may show its own selection or permission UI. Each new color sample refreshes the sampling canvas.

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

The repository does not include a system installer, release package, or application uninstall script. `cargo run` runs the app from the source checkout; `cargo build --release` creates a local binary under `target/release`. There is no separate system-wide installation to undo. `cargo clean` removes Cargo's build artifacts for this checkout.

Desktop Shortcut Maker is a feature within the app, not an installer for Mini Apps. It creates a launcher entry for the binary you select at `~/.local/share/applications/<name>.desktop` (the app name is sanitized for the filename). To remove an entry it created, delete that specific file from the applications directory.

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
