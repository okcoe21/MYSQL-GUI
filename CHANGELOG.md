# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2026-07-19

### Added
- Standalone **Tauri v2 Desktop Application** inside `desktop/` folder.
- Real-time Observability: Active connections, queries per second, and network traffic monitoring.
- Visual ER Diagrams: Interactive database relationship maps.
- Visual Query Builder: No-code selector for generating optimized queries.
- Smart Seeding: Built-in mock data generator for tables.
- Encrypted Session Layer: Local OS keychain integration via Rust `keyring` crate.
- Destruction Warnings: Dynamic alert systems for unsafe operations.
- Statement-Aware Importer: SQL script splitter with progress/error details.
- CI/CD Workflows: Automated builds for AppImage, deb, pacman, dmg, msi, and exe format.

---

## Roadmap

### Upcoming Features (v1.1.0)
- Support for SSH Tunneling and SSL database connections.
- Query performance profiling using `EXPLAIN ANALYZE`.
- Dark/Light mode theme syncing with system preferences.
- CSV/JSON bulk import utility.
