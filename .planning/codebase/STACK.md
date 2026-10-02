# Technology Stack

**Analysis Date:** 2026-10-03 (v3.0.0 Native Rust + Slint Rewrite)

## Languages

**Primary:**
- Rust (2021 edition) - All application backend, state management, controller, and database logic.
- Slint Markup (.slint) - Hardware-accelerated declarative UI markup and layout definitions.

## Runtime & Platforms

**Target Platforms:**
- Linux (X11 & Wayland)
- Windows (x86_64 MSVC)
- macOS (x86_64 & aarch64 Darwin)

**Runtime Overhead:**
- 100% Native binary execution.
- Zero Node.js, zero Chromium, zero WebViews. Sub-50ms cold startup.

**Package Manager & Build Tool:**
- Cargo (`Cargo.toml`, `Cargo.lock`)
- `build.rs` executing `slint-build` Ahead-Of-Time (AOT) compiler.

## Frameworks & Crates

**GUI Engine:**
- `slint` 1.18.1 - Native declarative UI framework with hardware acceleration (OpenGL/Skia/Software).
- `slint-build` 1.18.1 - Slint compile-time code generator.

**Async Runtime & Networking:**
- `tokio` 1.x (features: `full`) - Multi-threaded async runtime.

**Database Engine:**
- `sqlx` 0.8 (features: `runtime-tokio-native-tls`, `mysql`, `chrono`) - Non-blocking, connection-pooled async MySQL driver.

**Security & Key Management:**
- `keyring` 2.x (`keyring-rs`) - Direct integration with system credential managers (SecretService on Linux, Credential Manager on Windows, Keychain on macOS).

**Serialization & Utility:**
- `serde` 1.x (features: `derive`), `serde_json` 1.x - JSON data serialization for export and data mapping.
- `chrono` 0.4 (features: `serde`) - Timestamps and execution telemetry.
- `rand` 0.8 - Synthetic mock data generation.
- `dirs` 5.0 - Standard system directory discovery.
- `rfd` 0.15 - Native Rust File Dialogs for importing and exporting files.

## Developer & Tooling Dependencies

- `slint-viewer` 1.18.1 - Live interactive offline UI preview tool (`slint-viewer ui/app.slint`).
- `pmem` - Local workstation project memory catalog.
