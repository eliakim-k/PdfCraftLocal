# PdfCraftLocal 0.10.7-local.20261103

Prepared November 3, 2026; unpublished. Proposed tag: `v0.10.7-local.20261103`.

Desktop 0.10.7; installer 0.10.8. The executable is unchanged from the tested frozen baseline. This revision completes the source/asset notice payloads in the MSI, Setup and portable ZIP. PDF also includes LICENSE-document.txt beside the executable. Original component/font notices are retained in full, with source-supplied license texts appended.

Baseline source: `650ec11ea5c64f1955486d7b41946e5a6f069aec`. Executable SHA-256: `2bcbc13c6672866e853ea482fa8421605e0639ac46e6a03753e9436469260c6e`. The source import adds release documentation and packaging, removes upstream marketing marks/automatic workflows, and reconciles stale PDF attribution hashes against the unchanged baseline files. Application code, Cargo manifests, lockfile and runtime assets are unchanged.

The later Reader search increment is excluded. Drawn signatures and certificate-based signatures have different meanings.

The local editions provide explicit Preview + Send Q&A with a separately configured local runtime. Model weights and LM Studio are not included. Builds are unsigned. No complete Office/Reader equivalence is claimed.

Package validation: every final MSI file is extracted and compared with the expected payload; Setup must embed that exact MSI; portable files must match the same payload plus the portable marker. Compiled registry/shortcut policy is checked against tested baseline authoring, with no custom actions, no UserChoice writes and no automatic shutdown. Installer lifecycle was not replayed on the workstation. Historical application/installer tests remain retained and correctly scoped. Final scan and checksum receipts belong to the complete suite manifest.

The installer package version advances in the first three fields to permit upgrade from the previous package. Application version strings remain 0.10.7. Keep copies of important documents/settings and close the app before updating. Portable Data is private and excluded.

After publication, use only version-specific GitHub release URLs and verify SHA256SUMS.txt. The suite website will use those same files. No repository/tag/release has been published by local preparation.
