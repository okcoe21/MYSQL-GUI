# MySQL GUI — Antigravity Agent Rules

## Post-Iteration Maintenance Workflow (MANDATORY)

After EVERY feature change, bug fix, or update iteration completed in this project, you MUST automatically perform the following steps:

1. **Versioning:**
   - Bump version appropriately (SemVer) in `Cargo.toml`.
   - Update version in `README.md` (Title header and version badge).
   - Sync `Cargo.lock` by running `cargo check`.

2. **Automated Verification:**
   - Run `cargo check` and verify 0 errors, 0 warnings.
   - Run `cargo test --bin mysql-gui` and verify all tests pass.

3. **Documentation:**
   - Update `CHANGELOG.md` with the new version section.
   - Update `README.md` if any user-facing features or commands changed.
   - Update `CLAUDE_UPDATE_SUMMARY.md` with the latest status.
   - Keep `AUDIT.md` and `.planning/codebase/` in sync.

4. **Git Commit:**
   - Stage all updated files (`git add -A`).
   - Create a clean conventional commit.
   - Ensure the working tree is completely clean.
