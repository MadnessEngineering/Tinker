# Getting Started

## 1. Install native dependencies

Tinker builds on [`wry`](https://github.com/tauri-apps/wry) and
[`tao`](https://github.com/tauri-apps/tao), which bind the operating system's
own webview. That means there are system libraries to install before `cargo
build` will work — they are not in `Cargo.toml` and cargo cannot fetch them.

**This is the most common reason a fresh clone fails to build.**

### Linux (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install -y \
  build-essential pkg-config \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  libssl-dev
```

If you skip these, the build fails inside a dependency rather than in Tinker,
with a message that doesn't name the fix:

```
error: failed to run custom build command for `gdk-sys v0.18.2`
  > pkg-config --libs --cflags gdk-3.0 'gdk-3.0 >= 3.22'
  Package 'gdk-3.0', required by 'virtual:world', not found
```

That means `libgtk-3-dev` is missing. The equivalent failure naming
`webkit2gtk-4.1` means `libwebkit2gtk-4.1-dev` is missing.

On Fedora the packages are `gtk3-devel` and `webkit2gtk4.1-devel`; on Arch,
`gtk3` and `webkit2gtk-4.1`.

### macOS

Xcode Command Line Tools are enough — WebKit ships with the OS:

```bash
xcode-select --install
```

### Windows

Install [WebView2
Runtime](https://developer.microsoft.com/microsoft-edge/webview2/), which is
already present on current Windows 11. Build with the MSVC toolchain.

## 2. Build

```bash
cargo build
```

## 3. Run

```bash
# Open a URL
cargo run -- --url https://example.com

# With the REST + WebSocket API on port 3003
cargo run -- --api --url https://example.com

# As an MCP server for AI agents (JSON-RPC 2.0 over stdio)
cargo run -- --mcp --url https://example.com

# Both at once
cargo run -- --mcp --api --url https://example.com
```

### Command-line flags

| Flag | Description |
|---|---|
| `-u, --url <URL>` | URL to load |
| `-H, --headless` | Run without a visible window |
| `-b, --broker-url <URL>` | MQTT event broker URL |
| `--tabs <N>` | Number of tabs to open |
| `--record` | Enable event recording |
| `--record-path <PATH>` | Where to save recorded events |
| `--replay <PATH>` | Replay events from a file |
| `--replay-speed <N>` | Replay speed multiplier |
| `--api` | Enable the REST + WebSocket API server |
| `--api-port <PORT>` | API port (default `3003`) |
| `--mcp` | Enable the MCP server over stdio |
| `--no-restore-session` | Don't restore the previous session at startup |
| `--debug` | Debug logging |

## 4. Verify it works

```bash
cargo test
```

The Rust suites need no running browser. Some of them build and spawn the
`tinker` binary, so the first run takes a while.

For manual smoke tests against a live instance, see
[`tests/integration/README.md`](../tests/integration/README.md).

## Where to go next

- [MCP Server](mcp-server.md) — driving Tinker from an AI agent
- [Performance Monitoring](performance-monitoring.md) — Core Web Vitals, memory, profiling
- [Testing Guide](testing-guide.md) — writing tests against Tinker
- [Roadmap](../ROADMAP.md) — current status and what's planned

## Known rough edges

- **No CI yet.** Nothing automatically verifies builds across platforms; see M2
  in the [roadmap](../ROADMAP.md).
- **Cross-platform is unproven.** Development has been macOS-centric. Windows
  and Linux builds are expected to work but aren't regularly exercised.
