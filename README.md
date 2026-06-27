# skript-lsp

A [Language Server Protocol] 3.17 implementation for the [Skript] scripting
language (Minecraft server scripting). This is the **sidecar** that powers
[**Skript Studio**](https://github.com/skript-studio/skript-studio), a
standalone desktop IDE.

Skript syntax data is sourced live from the [SkriptHub] API.

[Language Server Protocol]: https://microsoft.github.io/language-server-protocol/
[Skript]: https://github.com/SkriptLang/Skript
[SkriptHub]: https://skripthub.net/

## Architecture

This is a Rust workspace with two crates:

| Crate | Type | Responsibility |
|-------|------|----------------|
| [`skript-core`](crates/skript-core) | lib | Pattern parsing, syntax index, completion, hover, validation, semantic tokens. Pure logic, no I/O. |
| [`skript-lsp`](crates/skript-lsp) | bin | The LSP server binary: WebSocket transport, `tower-lsp` wiring, SkriptHub fetch + cache, lifecycle. |

### Transport

The server speaks **LSP over WebSocket** — one JSON-RPC message per text or
binary frame, with no `Content-Length` framing on the wire. A small framing
adapter ([`ws.rs`](crates/skript-lsp/src/ws.rs)) translates between the
message-oriented WebSocket and the byte-stream codec that `tower-lsp::Server`
expects internally.

This transport was chosen so the Tauri frontend can connect from JavaScript
using the native `WebSocket` API, with no stdio plumbing.

### Lifecycle (the sidecar contract)

```
Tauri launcher                     skript-lsp
─────────────                      ───────────
spawn binary ──────────────►  parses args, binds 127.0.0.1:{port}
                              prints `LISTENING {port}` to stdout ◄─────┐
reads `LISTENING {port}` ──────────────────────────────────────────────┘
opens WS to ws://127.0.0.1:{port}
LSP initialize / initialized ──► spawns background SkriptHub fetch
…                                (data loads; server becomes `ready`)
LSP session runs (completions, hover, diagnostics, semantic tokens)
…
launcher kills process ◄──── on app exit (hard kill is safe)
```

**Stdout discipline:** the launcher parses stdout to discover the port, so
the *only* permitted stdout output is the single `LISTENING {port}` line.
All `tracing` logs go to **stderr**.

## Usage

### Build

```sh
cargo build --release -p skript-lsp
```

The binary is emitted at `target/release/skript-lsp[.exe]`.

### Run

```sh
# Bind an ephemeral port (launcher reads it from stdout)
skript-lsp --log-level info

# Bind a fixed port
skript-lsp --port 9876

# Use a cache dir for SkriptHub data (recommended for fast startup)
skript-lsp --cache-dir ~/.cache/skript-lsp

# Point at a different SkriptHub-compatible endpoint
skript-lsp --skripthub-url https://skripthub.net/api/v1/addonsyntaxlist/
```

#### CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--port <N>` | `0` (ephemeral) | TCP port to listen on. `0` asks the OS for a free port. |
| `--cache-dir <PATH>` | none | Directory for `skripthub-cache.json`. Enables instant startup from cache + background refresh. |
| `--skripthub-url <URL>` | `https://skripthub.net/api/v1/addonsyntaxlist/` | SkriptHub syntax-list endpoint. |
| `--log-level <LEVEL>` | `info` | One of `error`, `warn`, `info`, `debug`, `trace`. Overridable via `RUST_LOG`. |

### LSP capabilities advertised

- `textDocumentSync` — full document sync
- `completion` — Skript syntax completion (trigger char: space)
- `hover` — syntax docs rendered as Markdown
- `semanticTokens/full` — 9 token types (keyword, type, variable, string, comment, operator, event, function, number)
- `publishDiagnostics` — pushed on open/change and when the index becomes ready

All features degrade gracefully before the first SkriptHub fetch completes
(empty completions, `null` hover, deferred diagnostics).

## Development

```sh
# Type-check
cargo check

# Run unit tests
cargo test

# End-to-end smoke test (requires a running server on :9876)
cargo run -p skript-lsp -- --port 9876 --log-level info &
cargo test -p skript-lsp --test smoke -- --ignored --nocapture
```

## License

MIT.
