# Mod Manager (Rust Rewrite)

A Minecraft mod manager rewritten in Rust, originally based on [kaniol-lck/modmanager](https://github.com/kaniol-lck/modmanager) (C++/Qt).

## Tech Stack

- **Backend**: Rust + Tokio + Reqwest
- **Frontend**: Tauri 2 + React + TypeScript
- **API Support**: CurseForge, Modrinth, BMCLAPI (China mirror)

## Features

- Browse and search mods from CurseForge, Modrinth
- Download and install mods with version selection
- Local mod management (enable/disable, delete)
- Automatic update detection
- Minecraft version and mod loader filtering
- BMCLAPI mirror support for China users

## Project Structure

```
modmanager-rs/
├── Cargo.toml                    # Workspace config
├── crates/
│   ├── modmanager-core/          # Core library
│   │   └── src/
│   │       ├── api/              # API clients (CurseForge, Modrinth)
│   │       ├── models/           # Data models
│   │       ├── local/            # Local mod management
│   │       ├── download/         # Download engine
│   │       └── config/           # App configuration
│   └── modmanager-app/           # Tauri GUI app
│       ├── src/                  # Rust backend (Tauri commands)
│       └── tauri.conf.json       # Tauri configuration
└── frontend/                     # React frontend
    └── src/
        ├── App.tsx               # Main application
        └── styles.css            # Styling
```

## Architecture

### Core Library (modmanager-core)

The `ModPlatform` trait provides a unified interface for all mod platforms:

```rust
#[async_trait]
pub trait ModPlatform: Send + Sync {
    async fn search(&self, filter: &SearchFilter) -> Result<SearchResult>;
    async fn get_mod_info(&self, mod_id: &str) -> Result<ModInfo>;
    async fn get_mod_versions(&self, mod_id: &str) -> Result<Vec<ModVersion>>;
    // ...
}
```

Implemented by `CurseForgeApi` and `ModrinthApi`, with BMCLAPI as a CurseForge proxy.

### Design References

- **Prism Launcher**: Abstract ResourceAPI pattern, IndexedPack/IndexedVersion models, hash-based update detection, packwiz-compatible metadata
- **Original modmanager**: Feature set, UI layout, CurseForge/Modrinth integration

## Development

```bash
# Build core library only
cargo build -p modmanager-core

# Run tests
cargo test

# Dev mode (requires Node.js)
cd frontend && npm install
cd .. && cargo tauri dev
```

## License

GPL-3.0-only
