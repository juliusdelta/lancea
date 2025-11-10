# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture Overview

Lancea is a launcher application that consists of two main components:
- **Engine**: Rust-based D-Bus service (`lancea-engined`) that handles command resolution and provider management
- **UI**: Qt6/QML application that provides the graphical interface and communicates with the engine via D-Bus

### Engine Architecture (Rust)

The engine is organized as a Cargo workspace with the following crates:

- `lancea-model`: Core data structures and API models (Envelope, ResultItem, Preview, etc.)
- `lancea-bus`: D-Bus interface implementation and orchestration logic
- `lancea-registry`: Command resolution system that maps user input to providers
- `lancea-provider-emoji`: Emoji search provider with hardcoded emoji data
- `lancea-provider-apps`: Application launcher provider
- `lancea-engined`: Main executable that runs the D-Bus service

### UI Architecture (Qt6/QML)

- Qt6 application using QML for the interface
- `EngineProxy` (C++) provides D-Bus interface to QML
- Main UI components: search input, results list, preview pane
- Uses signal/slot pattern for D-Bus communication

### Communication Flow

1. User types in search input (e.g., "/emoji laugh")
2. QML calls `engineProxy.resolveCommand()` to determine provider
3. QML calls `engineProxy.search()` with resolved provider
4. Engine emits `ResultsUpdated` D-Bus signal with search results
5. EngineProxy receives signal and emits Qt signal to QML
6. QML updates results list and calls `engineProxy.requestPreview()` for selected item

## Common Development Commands

### Building

**Engine (Rust):**
```bash
cd engine
cargo build
cargo test
```

**UI (Qt6):**
```bash
cd ui
mkdir -p build && cd build
cmake ..
make
```

### Running

**Engine:**
```bash
cd engine
cargo run --bin engined
```

**UI:**
```bash
cd ui/build
./lancea-ui
```

### Testing

**Engine tests:**
```bash
cd engine
cargo test
cargo test --package lancea-engined -- dbus_roundtrip
```

The D-Bus roundtrip test verifies the complete communication flow between client and server.

## Key Implementation Details

### Provider System

- Providers implement search, preview, and execute functionality
- Each provider has a unique ID ("emoji", "apps")
- Registry resolves user input to determine which provider to use
- Providers return ResultItem structs with keys, titles, scores, and extras

### D-Bus Interface

The engine exposes `org.lancea.Engine1` interface with methods:
- `ResolveCommand(text_json) -> envelope(resolved_command)`
- `Search(args_json) -> token` (emits ResultsUpdated signal)
- `RequestPreview(args_json)` (emits PreviewUpdated signal)
- `Execute(args_json) -> envelope(outcome)`

### Command Resolution

Commands use slash prefixes:
- `/emoji` or `/em` → emoji provider
- `/apps` or `/ap` → apps provider
- Unmatched text → no provider (handled gracefully)

### Data Flow

All data is wrapped in `Envelope<T>` structures with versioning. Search results are batched into `ResultsBatch` enums (Reset/Insert/End) for efficient streaming.