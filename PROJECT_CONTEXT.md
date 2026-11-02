# QuadFurn CAD — Project Context & AI Agent Handover Guide

> **Purpose of this document**: This file provides a complete, self-contained technical and product summary of **QuadFurn CAD** so that any AI coding assistant or developer joining the project can immediately understand the architecture, engineering decisions, release workflow, current status (as of **v0.2.7**), and future roadmap without needing prior conversation history.

---

## 1. Selected Architecture & Technology Stack

### 1.1 Product Vision
**QuadFurn CAD** is a lightweight, high-performance, cross-platform CAD application for designing cabinet and panel furniture (made of laminated chipboard / LDSP, MDF, HDF, plywood) and generating production-ready cutting specifications and 2D guillotine sheet layouts.

The interaction paradigm combines three core concepts:
1. **Synchronized 4-Viewport Spatial Modeling**: Three 2D orthographic projections (**Top XY — Plan**, **Front XZ — Elevation**, **Side YZ — Section**) plus a real-time **3D Perspective Viewport** with both Orbit and First-Person Free-Fly (`Z` Noclip) camera modes.
2. **Direct 2D Box Manipulation & Precision Transform Bar**: Every furniture board/panel is represented as a 3D oriented bounding block (`PartBlock`) that can be created, selected, dragged, snapped, and resized via 8 perimeter handles in 2D orthographic views, or edited numerically via a top Transform Bar with inline arithmetic expression evaluation.
3. **Automated Manufacturing Pipeline**: Automatic net saw dimension calculation (subtracting PVC/ABS edge banding thickness on all 4 edges `L1`, `L2`, `W1`, `W2` from full finished dimensions), grouped Bill of Materials (Specification), multi-sheet 2D guillotine bin-packing optimizer, and a 4-section localized Vector PDF Production Passport exporter.

### 1.2 Technology Stack
- **Language**: **Rust** (2021 Edition) — 100% safe, single-codebase native + WebAssembly compilation.
- **GUI & 2D/3D Canvas Framework**: **`egui` / `eframe` 0.29** + **`egui_extras` 0.29**.
  - **Desktop Renderer**: `wgpu` backend (`AccessKit`, `X11`, `Wayland`, `DX12`/`Vulkan`/`Metal`).
  - **WebAssembly Renderer**: `glow` (WebGL2) backend compiled via **Trunk** (`wasm32-unknown-unknown`) and deployed as a zero-install progressive web application on GitHub Pages (`docs/app/`).
- **Linear Algebra & Geometry**: **`glam` 0.29** (`Vec2`, `Vec3`, `Vec4`, `Mat4` with `serde` serialization).
- **Serialization & Project Format**: **`serde` + `serde_json`** (`.qfurn` / `.json` human-readable UTF-8 project files with optional Base64 embedded textures).
- **Image & Texture Processing**: **`image` 0.25** (`PNG`, `JPEG`, `WEBP`) + **`base64` 0.22**.
- **Typography & PDF Font Parsing**: **`ab_glyph` 0.2** for TrueType (`.ttf`/`.ttc`) glyph metric inspection and custom CIDType2 (`Identity-H` + `/ToUnicode` CMap) subset embedding in generated PDF documents.
- **File Dialogs & Networking**:
  - Desktop: **`rfd` 0.15** for native OS file dialogs; **`ureq` 2.10** (with TLS) for non-blocking background GitHub release update checks.
  - Web (`wasm32`): **`web-sys` / `js-sys` / `wasm-bindgen`** for HTML5 File API uploads, Blob downloads, Canvas Pointer Lock API, clipboard integration, and on-demand dynamic font fetching.

### 1.3 Module & Source Tree Architecture
```text
QuadFurn/
├── .github/workflows/
│   └── build.yml                  # CI/CD matrix: Windows .zip, Linux .deb + .tar.gz, macOS Universal .app
├── .agents/skills/
│   └── release-checklist/         # Mandatory 3-phase release protocol instructions
├── assets/
│   ├── icon_256.jpg                  # Master application logo
│   └── icon.ico                   # Generated multi-resolution Windows icon
├── docs/                          # GitHub Pages landing site (index.html + 36 localized HTML pages)
│   ├── app/                       # Compiled WebAssembly CAD bundle (deployed via Trunk)
│   ├── README-*.md                # 36 localized Markdown documentation files
│   └── screenshots/               # Automated UI screenshots across all 37 supported languages
├── src/
│   ├── main.rs                    # App state, multi-product tab bar, Start Page, shortcuts, dialogs, Undo stack
│   ├── model.rs                   # Core domain models: Project, PartBlock, PartGroup, SheetMaterial, EdgeBanding, OBB
│   ├── viewport.rs                # 4-viewport controller, 2D ortho rendering, 3D painter pipeline, DAG depth sort
│   ├── snapping.rs                # Magnetic grid & part-edge snapping, 2D projection axis mappings
│   ├── cutter.rs                  # Specification aggregation, edge deduction, guillotine 2D bin-packing optimizer
│   ├── export.rs                  # 4-section A4 Landscape Vector PDF generator (CID TrueType), CSV & TSV exporters
│   ├── ui_panels.rs               # Top Transform Bar, Right Part Inspector, Specification, Cutting & Materials tabs
│   ├── i18n.rs                    # Localization dispatcher, OS locale detector, font loader for 37 languages
│   ├── glossary/                  # Modularized translation dictionaries by linguistic region:
│   │   ├── mod.rs                 # Glossary routing
│   │   ├── defaults.rs            # Localized default project, group, material, and part names
│   │   ├── formatters.rs          # Localized dynamic status strings and plural/parameterized messages
│   │   ├── slavic.rs              # RU, UK, BE, PL, CS, SK, SL, BG, SR, HR
│   │   ├── west_eu.rs             # DE, FR, ES, IT, PT, NL, HU, RO, EL, TR
│   │   ├── baltic_nordic.rs       # DA, NO, SV, FI, ET, LV, LT
│   │   ├── caucasus.rs            # KA, HY, AZ
│   │   ├── central_asia.rs        # KK, UZ
│   │   └── cjk.rs                 # ZH-CN, ZH-TW, JA, KO
│   ├── update_checker.rs          # Asynchronous GitHub Releases version checker (desktop)
│   ├── web.rs                     # WASM-specific helpers (file upload/download, pointer lock, dynamic font fetch)
│   ├── batch_screenshot.rs        # Automated headless/windowed screenshot generator for all 37 locales
│   └── bin/generate_changelog.rs  # CI helper for structured release notes generation
├── web/index.html                 # Trunk HTML template for the WebAssembly build
├── build.rs                       # Windows PE icon resource compiler
└── Cargo.toml                     # Package manifest (current version: 0.2.7)
```

---

## 2. Key Technical Decisions & Engineering Solutions

### 2.1 Spatial Representation & Part Orientation (`src/model.rs`)
- **World Coordinate System**: Millimeters (`mm`), right-handed:
  - `X` — Width (left to right)
  - `Y` — Depth (front to back)
  - `Z` — Height (bottom to top, floor at `Z = 0`)
- **Three Canonical Part Orientations (`PartOrientation`)**:
  1. `HorizontalXY` (*Shelf / Bottom / Roof*): `Length` along `X`, `Width` along `Y`, `Thickness` along `Z`.
  2. `VerticalYZ` (*Side Panel / Partition*): `Length` along `Z` (height), `Width` along `Y` (depth), `Thickness` along `X`.
  3. `FrontalXZ` (*Facade / Door / Back Wall*): `Length` along `Z` (height), `Width` along `X` (width), `Thickness` along `Y`.
- **Horizontal Rotation (`yaw_deg`)**: Every part supports rotation around the vertical `Z` axis (`∠Z`), allowing angled corner cabinets, diagonal doors, and rotated assemblies while maintaining an exact 3D Oriented Bounding Box (`OBB`).

### 2.2 3D Rendering Pipeline & Bulletproof Depth Sorting (`src/viewport.rs`)
Because `egui::Painter` renders 2D vector primitives and meshes without a hardware Z-buffer, custom 3D rendering required solving perspective projection, near-plane clipping, texture warping, and painter's depth sorting:
1. **3D Near-Plane Sutherland-Hodgman Clipping (`clip_3d_polygon_near_plane`)**:
   - Every 3D face quad and wireframe segment is clipped in 3D world space against the camera near plane (`NEAR_CLIP_MM = 12.0 mm`) *before* perspective division.
   - Prevents vertices behind or near the camera from dividing by near-zero `W` and stretching polygons to infinity across the screen.
2. **2D Screen-Rect Clipping (`clip_2d_polygon_rect`)**:
   - Projected polygons are clipped against `rect.expand(10.0)` using 2D Sutherland-Hodgman clipping, guaranteeing screen coordinates never overflow GPU vertex buffers when zooming in closely.
3. **Backface & Internal Abutment Culling**:
   - Back-facing faces (`world_norm.dot(eye - face_center) <= 0.0`) are culled immediately.
   - Narrow thickness edges (`18 mm` board edges) that are flush against an adjacent panel's AABB are culled when unselected so internal cabinet joints look clean without seam artifacts.
4. **Exact 2D Convex Polygon Intersection & Ray-Quad Topological Depth Sort**:
   - Parts are sorted back-to-front using **Kahn's Topological Sort** over a Directed Acyclic Graph (DAG) of part occlusion dependencies (with a farthest-depth tie-breaker and cycle-breaker).
   - To determine whether `Part A` occludes `Part B` (`does_part_a_occlude_part_b`), each pair of visible faces is intersected in 2D screen space via exact **Sutherland-Hodgman convex polygon clipping** (`clip_convex_polygon`).
   - If the 2D intersection polygon has a real interior area (`>= 2.0 sq px`), analytical 3D rays (`ray_from_screen_point`) are cast through the intersection centroid and interior sample points into both 3D face planes (`ray_quad_distance`). The face with smaller ray distance `t` is strictly in front.
   - **No Bounding-Box Fallback**: Parts that merely touch along a 1D edge or corner (such as adjacent doors, side panels meeting facades, or L-shaped countertop seams) have zero 2D interior overlap (`area == 0`), producing `None` (no false dependency edge in the DAG) and eliminating edge bleed-through and seam sorting errors.
5. **Dual 3D Render Quality Modes (`Enhanced` vs. `Normal`)**:
   - Controlled via the bottom-right toggle button in the 3D viewport (`cam_3d.z_order`, default `true` = `Enhanced`):
     - **`Enhanced` Mode**: Renders photorealistic woodgrain textures using perspective-correct 3D bilinear micro-tessellation (`3×3` up to `8×8` grid with symmetric X-split triangulation to eliminate affine texture warping) plus grain direction indicator lines.
     - **`Normal` Mode**: Bypasses all texture binding and micro-tessellation, rendering pure flat-shaded solid polygons with diffuse directional lighting (2 triangles per face) and clean wireframe edges — drastically reducing GPU load on integrated/low-end graphics hardware.

### 2.3 2D Orthographic Viewport Ergonomics (`src/viewport.rs`, `src/snapping.rs`)
- **Edge-On Thickness Protection**: When a board is viewed edge-on in a 2D projection (where one screen axis represents its `Thickness`, e.g., `18 mm`), corner resize handles are hidden and only the **2 length/width end handles** are active. This makes it impossible to accidentally drag and corrupt a board's material thickness when resizing in 2D.
- **Modifier Keys During 2D Dragging**:
  - `Shift + Drag`: Constrains movement strictly to the dominant horizontal or vertical axis.
  - `Ctrl / Cmd + Drag`: Bypasses magnetic grid and edge snapping for free placement.
  - `Alt + Drag`: Duplicates the selected part(s) at the drop location (leaving originals untouched).
- **Magnetic Snapping**: Snaps both to the configurable grid (`1`, `9`, `18`, `50 mm`) and to neighboring part edges/corners (`0 mm` flush abutment) with double snap attraction radius and live millimeter gap callouts.
- **Keyboard Nudging**: Arrow keys (`←`, `→`, `↑`, `↓`) nudge selected parts by the active grid step (`Shift + Arrow` nudges by `2×` grid step).
- **Layout-Independent Shortcuts**: All single-key and Ctrl/Cmd shortcuts (`V`, `B`, `M`, `Z`, `WASD`, `Ctrl+Z`, `Ctrl+S`, `Ctrl+C`, `Ctrl+V`, `Ctrl+D`) work regardless of the active OS keyboard layout (e.g., Cyrillic, Greek, CJK, or Latin) on both Desktop and WebAssembly.

### 2.4 Guillotine Sheet Cutting Optimizer (`src/cutter.rs`)
- **Default Saw Kerf Width**: Set to **`6.0 mm`** (`kerf_mm: 6.0`) as the industry standard for CNC router / panel sizing operations, with `10.0 mm` default 4-side sheet trim margins.
- **Net Saw Dimension Formula**:
  $$\text{Cut } L = \max(1.0,\; L_{\text{full}} - E_{L1} - E_{L2}), \quad \text{Cut } W = \max(1.0,\; W_{\text{full}} - E_{W1} - E_{W2})$$
- **Grain Direction & Cross-Grain Support**:
  - `grain_fixed: true` locks part orientation to the sheet length axis.
  - `grain_along_width: true` rotates the woodgrain direction $90^\circ$ along the part's width (both in 3D UV mapping and in the guillotine packer).
- **Heuristic Guillotine Packing**: Best-Area-Fit / Best-Short-Side-Fit guillotine split with free-rectangle merging, remnant tracking, linear cut length calculation, and $+15\%$ edge banding waste estimation.

### 2.5 Multi-Script Typography & Vector PDF Export (`src/export.rs`, `src/i18n.rs`)
- **37 Supported Languages**:
  `EN`, `RU`, `UK`, `BE`, `KK`, `KA`, `HY`, `AZ`, `UZ`, `DE`, `FR`, `ES`, `IT`, `PT`, `PL`, `CS`, `SK`, `SL`, `BG`, `SR`, `HR`, `NL`, `DA`, `NO`, `SV`, `FI`, `ET`, `LV`, `LT`, `HU`, `RO`, `TR`, `EL`, `ZH-CN`, `ZH-TW`, `JA`, `KO`.
- **Dynamic Web Font Loading (`src/web.rs`)**:
  - To keep the initial WASM bundle small and fast, specialized script fonts (CJK, Georgian, Armenian, etc.) are fetched asynchronously on demand from Google Fonts CDN when the user switches to those languages in the browser.
- **TrueType CID PDF Embedding**:
  - Generates multi-page A4 Landscape vector PDFs without external PDF crates, embedding raw TrueType font tables (`Type0` / `CIDFontType2` with `Identity-H` encoding and `/ToUnicode` CMap) so exported PDFs render and copy-paste cleanly in all 37 languages on both Desktop and WebAssembly.

---

## 3. Current Roadmap Status & Completed Milestones (Up to v0.2.7)

| Version | Status | Key Deliverables Completed |
| :--- | :---: | :--- |
| **v0.1.0** | ✅ Done | Core 4-viewport CAD, 3D orbit/fly camera, OBB raycasting, inline math expressions, edge banding deduction, guillotine 2D nesting, Vector PDF & CSV export. |
| **v0.2.0** | ✅ Done | Texture manager (`PNG`/`JPG`/`WEBP`), Base64 project texture embedding, relative path fallbacks, initial 22-language UI & documentation localization. |
| **v0.2.1 – v0.2.2** | ✅ Done | macOS `Cmd` shortcut adaptations, dynamic cross-platform font resolution, 2D keyboard arrow nudging (`Shift` 2× step), GitHub Actions release matrix. |
| **v0.2.3** | ✅ Done | Full WebAssembly (WASM) port via Trunk on GitHub Pages, interactive Missing Textures Resolver dialog, TrueType CID PDF export in browser. |
| **v0.2.4** | ✅ Done | Start Page on launch, `Shift`/`Ctrl`/`Alt` 2D drag modifiers, edge-on handle protection, viewport equalization (`⊞`), dynamic CJK web fonts, topological 3D sort. |
| **v0.2.5** | ✅ Done | Expansion to **37 languages**, modularized `src/glossary/` architecture, 3-column language selector, automated screenshot generation across all 37 locales. |
| **v0.2.6** | ✅ Done | Background GitHub update checker with clickable top-bar notification badge, Web Pointer Lock & WASD fly mode fixes, perspective-correct 3D textures. |
| **v0.2.7 (Current)** | ✅ Done | **6.0 mm default saw kerf**; 3D near-plane & 2D screen clipping; exact convex polygon intersection ray-quad DAG depth sorting; **`Enhanced` / `Normal` 3D render quality modes**. |

---

## 4. Deferred Features, Open Ideas & Future Opportunities

The following ideas, limitations, and potential enhancements were discussed or identified during development and are deferred for future iterations:

1. **Per-Viewport Line Anti-Aliasing (Feathering) Control in `egui`**:
   - *Context*: We explored disabling `egui::epaint::TessellationOptions::feathering` in `Normal` 3D mode to render 100% crisp, un-feathered vector lines. However, in `egui 0.29`, `feathering` is a global per-frame context setting — turning it off in the 3D pane also disables anti-aliasing for all UI text, icons, and 2D viewports.
   - *Future Solution*: Either render the 3D viewport into an offscreen `wgpu`/`glow` texture/framebuffer with independent MSAA/raster state, or construct 1px screen-aligned quad strips manually in a custom `egui::PaintCallback`.
2. **Hardware Depth Buffer (`wgpu` / `WebGL` Native 3D Pass via `egui::PaintCallback`)**:
   - *Context*: The current 3D viewport uses a software-sorted painter's pipeline (`egui::Painter` meshes + Kahn's topological sort with exact 2D convex polygon intersection ray-casting). It handles all standard non-interpenetrating cabinet assemblies cleanly, except for true 3D interpenetrating geometry or coplanar Z-fighting surfaces.
   - *Future Solution*: If complex intersecting meshes or hardware-imported 3D models (`.obj` / `.stl` / `.gltf` handles, hinges, or appliances) are added in the future, moving the 3D pane to a custom `egui::PaintCallback` with a native 24-bit hardware depth buffer (`Depth24Plus`) will resolve interpenetrating geometry at the pixel level.
3. **Parametric Hardware, Drilling & Fastener System (Minifix, Confirmats, Hinges, Drawer Slides)**:
   - Adding drilling markers, hinge cup cutouts (`35 mm`), dowel/confirmat hole patterns, and automatic hardware count tables in the Production Passport.
4. **Parametric Assembly Templates & Smart Resizing**:
   - Expanding the Start Page template library (kitchen base cabinets, wall cabinets, corner L-cabinets, sliding wardrobes, drawer boxes) with parametric overall width/height/depth resizing that automatically recalculates internal shelf and door dimensions.
5. **DXF / G-Code / CNC Post-Processor Export**:
   - Exporting individual part contours and cutting layouts to `.dxf` or CNC Nesting formats alongside the existing Vector PDF and Excel CSV exports.

---

## 5. Mandatory Project Rules & Release Protocol (For AI Agents)

1. **Strict Terminology Policy**:
   - **NEVER** mention proprietary commercial CAD, vector graphics, or game level-editor software names in code comments, commit messages, documentation, or chat responses. Always describe workflows using generic, functional engineering terminology.
2. **Code Comments & Documentation Integrity**:
   - All source code comments and docstrings must remain in **English**.
   - Whenever features, UI controls, or defaults change, synchronize `README.md` and all 36 localized `docs/README-*.md` files.
3. **Mandatory 3-Phase Release Workflow (`AGENTS.md`)**:
   - **Phase 1**: Run `cargo test --release`, `cargo check --target wasm32-unknown-unknown`, and `cargo clippy --all-targets`. Bump version in `Cargo.toml`, commit, push `main`, create annotated tag `vX.Y.Z`, and push tag.
   - **Phase 2**: Wait for the GitHub Actions release workflow on tag `vX.Y.Z` to succeed across all targets and verify that all 4 binary assets (`QuadFurn-windows-x86_64.zip`, `quadfurn_X.Y.Z_amd64.deb`, `QuadFurn-linux-x86_64.tar.gz`, `QuadFurn-macos-universal.zip`) are published on GitHub Releases.
   - **Phase 3**: Rebuild the WebAssembly app (`trunk build web/index.html --release --dist docs/app --public-url ./`), update version numbers across `README.md`, `docs/README-*.md`, and `docs/*.html`, commit, and push to `main` so GitHub Pages deploys the updated web application and documentation.
