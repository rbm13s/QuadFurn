# QuadFurn CAD — Agent Guidelines & Rules

## 1. Release Workflow & Pre-Release Checklist Rule

Before releasing a new version, ALWAYS follow this mandatory three-phase release protocol:
**First push the tag, wait for GitHub Actions desktop builds and release assets to succeed, and only then build & deploy the WebAssembly version and update READMEs and HTML pages.**

### Phase 1: Local Preflight & Version Tagging
1. **Local Preflight Checks**:
   - Run `cargo test --release` — verify that all unit tests pass with zero failures.
   - Run `cargo check --target wasm32-unknown-unknown` — verify that WebAssembly compilation succeeds without errors.
   - Run `cargo clippy --all-targets` — ensure there are no breaking or critical warnings.
2. **Version Bump in Cargo.toml**:
   - Update `version = "X.Y.Z"` in `Cargo.toml`.
   - Commit the version bump:
     `git commit -am "chore(release): bump version to vX.Y.Z"`
   - Push commit to `main`: `git push origin main`.
3. **Tag & Push Release**:
   - Create an annotated git tag:
     `git tag -a vX.Y.Z -m "Release vX.Y.Z - <Summary of changes>"`
   - Push the tag to remote:
     `git push origin vX.Y.Z`

### Phase 2: GitHub Actions CI Build Verification
1. **Monitor Release Pipeline**:
   - Query GitHub Actions workflow status for the release run triggered on tag `vX.Y.Z`.
   - Wait until all desktop matrix build jobs succeed:
     - `Windows (x86_64)`
     - `Linux (Debian / Ubuntu x86_64)`
     - `macOS Universal (Apple Silicon + Intel)`
     - `Publish GitHub Release`
2. **Verify Published Release Assets**:
   - Query GitHub Releases API to verify all 4 binary packages are published and attached:
     - `QuadFurn-windows-x86_64.zip`
     - `quadfurn_X.Y.Z_amd64.deb`
     - `QuadFurn-linux-x86_64.tar.gz`
     - `QuadFurn-macos-universal.zip`
   - **NEVER** proceed to Phase 3 if any desktop build failed or any asset is missing.

### Phase 3: WebAssembly CAD Rebuild & Documentation Synchronization
*Only after Phase 2 passes and all binary assets are verified on GitHub Releases:*
1. **Rebuild WebAssembly (WASM) Online CAD for GitHub Pages**:
   - Run `trunk build web/index.html --release --dist docs/app --public-url ./` (or `.\build_wasm.ps1` / `./build_wasm.sh`).
   - Verify that `docs/app/` contains the newly compiled `.wasm` and `.js` bundle for `vX.Y.Z`.
2. **Synchronize All Markdown Documentation**:
   - Primary `README.md` at root.
   - All 21 localized documentation files in `docs/`:
     `docs/README-ru.md`, `docs/README-uk.md`, `docs/README-de.md`, `docs/README-pl.md`,
     `docs/README-es.md`, `docs/README-fr.md`, `docs/README-it.md`, `docs/README-pt.md`,
     `docs/README-cs.md`, `docs/README-sk.md`, `docs/README-sl.md`, `docs/README-nl.md`,
     `docs/README-da.md`, `docs/README-no.md`, `docs/README-sv.md`, `docs/README-hu.md`,
     `docs/README-ro.md`, `docs/README-tr.md`, `docs/README-zh-cn.md`, `docs/README-ja.md`,
     `docs/README-ko.md`.
3. **Synchronize All HTML Pages**:
   - Update version badges (e.g. `<span class="badge-ver">vX.Y.Z</span>`), hero banners, and download links in `docs/index.html` and all 21 localized HTML files (`docs/ru.html`, `docs/de.html`, etc.).
   - Verify that all platform download URLs point to the newly published release.
4. **Commit & Push to Main**:
   - Commit the WebAssembly build and synchronized documentation:
     `git commit -am "docs(release): sync documentation and deploy web cad for vX.Y.Z"`
   - Push to `main`: `git push origin main`.
   - Monitor `pages build and deployment` on GitHub Actions until it finishes with status `success`.

---

## 2. Documentation Synchronization Rule
Whenever any functionality, UI controls, hotkeys, algorithms, or architecture in QuadFurn are added or modified:
1. Update `README.md` and all 21 localized documentation files in `docs/README-*.md`.
2. Maintain cross-linking navigation between all languages at the top of every README file.
3. Maintain documentation integrity and ensure all code comments across the codebase remain in English.
