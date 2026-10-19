---
name: release-checklist
description: >-
  Enforce the mandatory 3-phase release procedure for QuadFurn: verify all CI builds on GitHub Actions pass, synchronize version numbers across Cargo.toml, READMEs, and HTML files, and finally tag and publish the release. Use whenever preparing, cutting, bumping, or deploying a new release or version tag.
---

# Release Checklist & Workflow for QuadFurn

This skill provides step-by-step guidance for safely releasing a new version of QuadFurn.

## Phase 1: Preflight Verification

Before touching version numbers or creating tags, verify build integrity locally and in CI:

1. **Local Test & Build Verification**:
   ```powershell
   cargo test --release
   cargo check --target wasm32-unknown-unknown
   cargo clippy --all-targets
   ```
   All tests must pass, with 0 failures and 0 errors.

2. **GitHub Actions Verification**:
   Query GitHub Actions runs on `main` using PowerShell:
   ```powershell
   Invoke-RestMethod -Uri "https://api.github.com/repos/rbm13s/QuadFurn/actions/runs?per_page=3" |
     Select-Object -ExpandProperty workflow_runs |
     Select-Object id, status, conclusion, head_branch, name
   ```
   Verify that the latest commit on `main` succeeded on:
   - Windows (x86_64)
   - Linux (Debian / Ubuntu x86_64)
   - macOS Universal (Apple Silicon + Intel)
   - GitHub Pages deploy

> [!CAUTION]
> If any job has failed or is still in progress, resolve all issues first. NEVER create a release tag from an unverified or failing commit.

---

## Phase 2: Version & Documentation Synchronization

1. **Update `Cargo.toml`**:
   Set `version = "X.Y.Z"`.

2. **Update Markdown Documentation**:
   - Primary `README.md`
   - All 21 localized files in `docs/README-*.md`
   Ensure badges, headers, and links reflect `vX.Y.Z`.

3. **Update HTML Website Pages**:
   - `docs/index.html` and all 21 localized HTML files in `docs/*.html`.
   - Update:
     - Version badge: `<span class="badge-ver">vX.Y.Z</span>`
     - Hero banner: `vX.Y.Z Available • ...`
     - Section heading: `Download QuadFurn CAD vX.Y.Z`
     - Download URLs for Windows, Linux (.deb, .tar.gz), and macOS.

4. **Commit & Push to `main`**:
   ```powershell
   git commit -am "chore(release): bump version to vX.Y.Z and sync documentation"
   git push origin main
   ```
   Wait for the GitHub Actions push workflow on `main` to pass.

---

## Phase 3: Tagging & Release Publishing

1. **Create Annotated Tag**:
   ```powershell
   git tag -a vX.Y.Z -m "Release vX.Y.Z - <Release Title and Summary>"
   ```

2. **Push Tag to GitHub**:
   ```powershell
   git push origin vX.Y.Z
   ```

3. **Monitor Release Build**:
   ```powershell
   Invoke-RestMethod -Uri "https://api.github.com/repos/rbm13s/QuadFurn/actions/runs?per_page=1" |
     Select-Object -ExpandProperty workflow_runs |
     Select-Object id, status, conclusion
   ```
   Ensure all matrix jobs and `Publish GitHub Release` finish with `conclusion: success`.

4. **Verify GitHub Release Assets**:
   ```powershell
   Invoke-RestMethod -Uri "https://api.github.com/repos/rbm13s/QuadFurn/releases/tags/vX.Y.Z" |
     Select-Object -ExpandProperty assets |
     Select-Object name, size
   ```
   Confirm presence of:
   - `QuadFurn-windows-x86_64.zip`
   - `quadfurn_X.Y.Z_amd64.deb`
   - `QuadFurn-linux-x86_64.tar.gz`
   - `QuadFurn-macos-universal.zip`
