# ccboard-web

Web frontend for ccboard using Leptos (CSR) + Axum backend.

## Architecture

- **Backend**: Axum server serving API endpoints + HTML shell
- **Frontend**: Leptos CSR app compiled to WASM
- **Router**: Leptos Router for SPA navigation
- **Build**: Trunk for WASM compilation

## Development

### Prerequisites

Install Trunk for WASM builds:

```bash
cargo install trunk
rustup target add wasm32-unknown-unknown
```

### Build WASM

```bash
cd crates/ccboard-web
trunk build --release
```

This generates:
- `dist/index.html` - HTML shell with WASM loader
- `dist/ccboard-web-*.wasm` - Compiled WASM binary
- `dist/ccboard-web-*.js` - WASM bindings

### Run Development Server

Option 1: Trunk dev server (frontend only, mocked API):

```bash
cd crates/ccboard-web
trunk serve
# Opens http://localhost:8080
```

Option 2: Full stack (Axum backend + WASM frontend):

```bash
# Build WASM first
cd crates/ccboard-web && trunk build --release && cd ../..

# Run Axum server
cargo run -- web --port 3333
# Opens http://localhost:3333
```

## Structure

```
ccboard-web/
├── src/
│   ├── app.rs           # Main App component + Router
│   ├── components/      # Reusable UI components
│   │   ├── header.rs
│   │   └── sidebar.rs
│   ├── pages/           # Page components
│   │   ├── dashboard.rs
│   │   ├── sessions.rs
│   │   └── analytics.rs
│   ├── router.rs        # Axum backend routes
│   ├── sse.rs           # Server-Sent Events
│   ├── lib.rs           # Library exports
│   └── main.rs          # WASM entry point
├── index.html           # Trunk template
├── Trunk.toml           # Trunk config
└── static/              # Static assets
```

## Routes

### Frontend (SPA)

- `/` - Dashboard
- `/sessions` - Sessions Explorer
- `/analytics` - Analytics
- `/config` - Config (stub)
- `/history` - History (stub)

### Backend (API)

- `/api/stats` - Stats JSON
- `/api/sessions` - Sessions JSON
- `/api/health` - Health check
- `/api/events` - SSE stream

## Status (W1.1 Complete)

- ✅ Leptos App structure
- ✅ SPA Router with 5 routes
- ✅ Header + Sidebar components
- ✅ Page stubs (Dashboard, Sessions, Analytics)
- ✅ Axum backend serving HTML shell
- ✅ Build configuration (Trunk + Cargo)

## Next Steps (W1.2)

- Fetch `/api/stats` on Dashboard mount
- Display stats cards with real data
- Add loading states + error handling
