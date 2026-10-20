---
name: release-checklist
description: >-
  Enforce the mandatory 3-phase release procedure for QuadFurn: verify all CI builds on GitHub Actions pass, synchronize version numbers across Cargo.toml, READMEs, and HTML files, and finally tag and publish the release. Use whenever preparing, cutting, bumping, or deploying a new release or version tag.
---

# Release Checklist & Workflow for QuadFurn

This skill provides step-by-step guidance for safely releasing a new version of QuadFurn.

**Mandatory rule: First push the version tag, wait for GitHub Actions desktop builds and release assets to succeed, and only then build & deploy the WebAssembly version and update READMEs and HTML pages.**

---

## Phase 1: Local Preflight & Version Tagging

1. **Local Test & Build Verification**:
   ```powershell
   cargo test --release
   cargo check --target wasm32-unknown-unknown
   cargo clippy --all-targets
   ```
   All tests must pass with 0 failures and 0 errors.

2. **Bump Version in `Cargo.toml`**:
   Set `version = "X.Y.Z"`.

3. **Commit & Push Version Bump**:
   ```powershell
   git commit -am "chore(release): bump version to vX.Y.Z"
   git push origin main
   ```

4. **Create Annotated Tag & Push**:
   ```powershell
   git tag -a vX.Y.Z -m "Release vX.Y.Z - <Release Title and Summary>"
   git push origin vX.Y.Z
   ```

---

## Phase 2: GitHub Actions CI Build Verification

1. **Monitor Release Pipeline**:
   Query GitHub Actions runs for tag `vX.Y.Z` using PowerShell:
   ```powershell
   Invoke-RestMethod -Uri "https://api.github.com/repos/rbm13s/QuadFurn/actions/runs?per_page=3" |
     Select-Object -ExpandProperty workflow_runs |
     Select-Object id, status, conclusion, head_branch, name
   ```
   Wait until all matrix targets succeed:
   - Windows (x86_64)
   - Linux (Debian / Ubuntu x86_64)
   - macOS Universal (Apple Silicon + Intel)
   - Publish GitHub Release

2. **Verify Published Release Assets**:
   ```powershell
   (Invoke-RestMethod -Uri "https://api.github.com/repos/rbm13s/QuadFurn/releases/tags/vX.Y.Z").assets |
     Select-Object name, size
   ```
   Ensure all 4 binary packages are attached:
   - `QuadFurn-windows-x86_64.zip`
   - `quadfurn_X.Y.Z_amd64.deb`
   - `QuadFurn-linux-x86_64.tar.gz`
   - `QuadFurn-macos-universal.zip`

> [!CAUTION]
> NEVER proceed to Phase 3 if any desktop build failed or if release assets are missing or unverified.

---

## Phase 3: WebAssembly CAD Rebuild & Documentation Synchronization

*Only after Phase 2 passes and all binary assets are verified on GitHub Releases:*

1. **Rebuild WebAssembly (WASM) Online CAD for GitHub Pages**:
   ```powershell
   trunk build web/index.html --release --dist docs/app --public-url ./
   ```
   Verify that `docs/app/` contains the updated `.wasm` and `.js` bundle for `vX.Y.Z`.

2. **Update Markdown Documentation**:
   - Primary `README.md` at root
   - All 21 localized files in `docs/README-*.md`
   Ensure badges, headers, and links reflect `vX.Y.Z`.

3. **Update HTML Website Pages**:
   - `docs/index.html` and all 21 localized HTML files in `docs/*.html`.
   - Update:
     - Version badge: `<span class="badge-ver">vX.Y.Z</span>`
     - Hero banner: `vX.Y.Z Available • ...`
     - Section heading: `Download QuadFurn CAD vX.Y.Z`
     - Platform download links pointing to the published release assets.

4. **Commit & Push to `main`**:
   ```powershell
   git add docs/app/
   git commit -am "docs(release): sync documentation and deploy web cad for vX.Y.Z"
   git push origin main
   ```
   Wait for `pages build and deployment` on GitHub Actions to succeed.
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
