# QuadFurn CAD — Agent Guidelines & Rules

## 1. Release Workflow & Pre-Release Checklist Rule

Before creating or pushing any new release version tag (e.g. `vX.Y.Z`) to the repository, ALWAYS follow this mandatory three-phase release protocol:

### Phase 1: CI Build Verification & Local Preflight
1. **Local Preflight Checks**:
   - Run `cargo test --release` — verify that all unit tests pass with zero failures.
   - Run `cargo check --target wasm32-unknown-unknown` — verify that WebAssembly compilation succeeds without errors.
   - Run `cargo clippy --all-targets` — ensure there are no breaking or critical warnings.
2. **GitHub Actions CI Verification**:
   - Query GitHub Actions workflow status for the latest commit on `main`.
   - Ensure that all 3 desktop matrix targets (Windows x86_64, Linux Debian/Ubuntu x86_64, macOS Universal) and the Pages deployment have passed with status `success`.
   - **NEVER** push a release tag if any CI job on `main` is failed, in progress, or unverified.

### Phase 2: Version & Documentation Synchronization
Only after Phase 1 passes:
1. **Bump Version in Cargo.toml**: Update `version = "X.Y.Z"`.
2. **Synchronize All Documentation**:
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
   - Verify that all platform download URLs point to the upcoming release:
     - `https://github.com/rbm13s/QuadFurn/releases/download/vX.Y.Z/QuadFurn-windows-x86_64.zip`
     - `https://github.com/rbm13s/QuadFurn/releases/download/vX.Y.Z/quadfurn_X.Y.Z_amd64.deb`
     - `https://github.com/rbm13s/QuadFurn/releases/download/vX.Y.Z/QuadFurn-linux-x86_64.tar.gz`
     - `https://github.com/rbm13s/QuadFurn/releases/download/vX.Y.Z/QuadFurn-macos-universal.zip`
4. **Rebuild WebAssembly (WASM) Online CAD for GitHub Pages**:
   - Run `trunk build web/index.html --release --dist docs/app --public-url ./` (or `.\build_wasm.ps1` / `./build_wasm.sh`).
   - Verify that `docs/app/` contains the newly compiled `.wasm` and `.js` bundle.
5. **Commit & Push to Main**:
   - Commit the version bump, documentation sync, and WASM web build:
     `git commit -am "chore(release): bump version to vX.Y.Z and sync documentation"`
   - Push to `main`: `git push origin main`.
   - Wait for the GitHub Actions push build on `main` to pass successfully.

### Phase 3: Tagging and Releasing
Only after Phase 1 and Phase 2 are verified:
1. **Tag the Release**:
   - Create an annotated git tag:
     `git tag -a vX.Y.Z -m "Release vX.Y.Z - <Summary of changes>"`
2. **Push the Tag**:
   - `git push origin vX.Y.Z`
3. **Monitor Release Pipeline**:
   - Monitor the triggered GitHub Actions release workflow run until all build jobs (Windows, Linux, macOS) and the `Publish GitHub Release` job succeed.
   - Verify that all 4 binary assets are attached to the release on GitHub.

---

## 2. Documentation Synchronization Rule
Whenever any functionality, UI controls, hotkeys, algorithms, or architecture in QuadFurn are added or modified:
1. Update `README.md` and all 21 localized documentation files in `docs/README-*.md`.
2. Maintain cross-linking navigation between all languages at the top of every README file.
3. Maintain documentation integrity and ensure all code comments across the codebase remain in English.
