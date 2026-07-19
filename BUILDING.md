# Building MySQL GUI Desktop

This document guides you on building and releasing the Tauri-based desktop app locally and via CI/CD.

## Local Build Instructions

First, ensure you have the prerequisite tools installed (Node.js, Rust, Cargo, and system build tools).

1. Change directory to the desktop subproject:
   ```bash
   cd desktop
   ```
2. Install dependencies:
   ```bash
   npm install
   ```
3. Run the development server (to launch the live desktop client window):
   ```bash
   npm run tauri dev
   ```
4. Compile for production:
   ```bash
   npm run tauri build
   ```
   The compiled binaries will be generated inside `desktop/src-tauri/target/release/bundle/`.

---

## Prerequisites per Platform

### Linux (Ubuntu/Debian)
Install these development libraries:
```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev build-essential curl wget
```

### macOS
Install the Xcode Command Line Tools:
```bash
xcode-select --install
```

### Windows
Install Visual Studio Community (or Microsoft C++ Build Tools) with the "Desktop development with C++" workload selected.

---

## Release Pipeline (CI/CD)

Releases are fully automated via GitHub Actions:
1. When you push a semantic version tag (e.g., `v0.1.0`):
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
2. The `Release Tauri App` workflow triggers automatically.
3. It compiles the Tauri application for:
   - **Linux** (`.AppImage`, `.deb`, `.pacman`)
   - **macOS** (`.dmg`)
   - **Windows** (`.msi`, `.exe`)
4. A draft GitHub Release is created, and all output binaries are attached to the release assets.
