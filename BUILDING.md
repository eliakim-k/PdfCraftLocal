# Build and package PdfCraftLocal

The distributed executable was built from baseline commit `650ec11ea5c64f1955486d7b41946e5a6f069aec`, with SHA-256 `2bcbc13c6672866e853ea482fa8421605e0639ac46e6a03753e9436469260c6e`. This public snapshot retains its application source, Cargo files and runtime assets, and adds documentation/packaging metadata. PDF attribution records now match the retained asset bytes. A fresh build from this import has not been claimed byte-identical to the baseline.

The retained native recipe uses Rust 1.95.0, `x86_64-pc-windows-msvc`, MSVC 14.44 and Windows SDK 10.0.26100. Source build scripts and locked dependencies should be reviewed before execution. Fetch the locked public dependencies separately, then build offline in an x64 developer environment:

```powershell
$env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS='-C target-feature=+crt-static'
$env:PDFCRAFT_REQUIRE_WINRES='1'
$env:PDFCRAFT_BUILD_SHA=(git rev-parse HEAD)
cargo +1.95.0 build --release --locked --offline --target x86_64-pc-windows-msvc -p pdfcraft
```

Keep the desktop local-only policy, input budgets and strict icon/version resources. Rebuilding changes provenance and requires its own validation. The retained minimized baseline build recipes clear the environment and do not set CRAFT_FONTS_DIR; optional craft-fonts are not enabled by those recipes. Source fonts and dependency-embedded fonts retain their separate licenses.

For this notice-only revision, packaging reuses the original verified executable. The exact WiX authoring and payload documentation are under packaging/release/. Packaging version 0.10.8 differs deliberately from desktop 0.10.7. Per-user installer behavior and upgrade GUIDs are preserved; document notices are added. No compiler, runtime, model, private cache or test profile is shipped. See packaging/release/BUILD.md for the wrapper commands.

Historical app CI/test/startup/lifecycle receipts are not new executions. Final wrapper verification uses extraction/byte comparisons and compiled policy checks, without installing apps. No native UI or model inference is part of package validation. New application features require a separately validated release.
