# Integration Scripts

These Python scripts exercise a **running** Tinker instance over its REST API or
MCP stdio interface. They are not part of `cargo test` and nothing runs them
automatically — they're manual smoke tests.

For the Rust unit and integration suites, use `cargo test`. Those need no
running browser.

## Prerequisites

```bash
pip install requests websockets
```

## Running the API scripts

All of these expect the API server on `127.0.0.1:3003`. Start it first, in a
separate terminal:

```bash
cargo run -- --api --url https://example.com
```

Then:

| Script | Exercises |
|---|---|
| `test_api.py` | Core endpoints plus the WebSocket at `/ws` (needs `websockets`) |
| `test_dom_simple.py` | Minimal DOM find/interact smoke test |
| `test_dom_inspector.py` | Selectors, interaction, highlighting, wait conditions |
| `test_network_monitoring.py` | Network capture, filters, HAR export |
| `test_visual_api.py` | Screenshots, baselines, visual comparison |
| `test_recording_replay.py` | Recording and playback endpoints |

## Running the MCP script

`test_mcp_server.py` is the exception — it spawns its own Tinker process and
speaks JSON-RPC 2.0 over stdio, so don't start a server first:

```bash
python tests/integration/test_mcp_server.py
```

## Caveats

These scripts predate the current test suite and overlap with it in places.
They assert loosely and print rather than fail hard, so read the output rather
than trusting the exit code. Treat them as diagnostic tools, not a gate.
