> Historical build/edition notes. For the current package identity, behavior, licenses and assurance scope, use README.md, BUILDING.md and packaging/release/BUILD.md. Earlier paths and deferred-feature statements below describe their original revision.

# PdfCraft Local

Independent Windows build. Originals in D:/SOFTWARE/ArtCraft/pdfcraft remain intact.

Built-in: interactive AcroForm filling and creation, checkboxes and choices; Fill & Sign for
flat pages; drawn, typed and imported image signatures; certificate-based digital signing;
highlights, underlines, strikethrough, notes, comment replies and review status; page
organization, text editing, search, redaction, printing and export. Availability is not a
claim of complete Acrobat compatibility. Drawn/image signatures are visual marks, not
cryptographic authentication. Digital signature validity depends on certificate trust.

This build disables PDF JavaScript even if preferences request it, URL dispatch, update
requests, the UI automation listener and single-instance IPC. Forms requiring custom
JavaScript calculations or XFA script logic may need manual values. Script objects are
preserved on save, not automatically sanitized, to avoid invalidating signed documents.
Use a separate sanitized copy when sharing untrusted PDFs with other readers.

Reads are capped at 128 MiB at the GUI file boundary. Direct UNC, device, URL and alternate
data-stream paths are rejected. This is not an OS sandbox: mapped drives, junctions,
printers, clipboard and the file picker retain Windows behavior. Complex or malicious
PDFs can still exhaust time or resources; no review proves software free of defects.

Portable settings, recovery and IDs live under Data beside the executable. Installed
settings live under %LOCALAPPDATA%/PdfCraftLocal. Keep private digital IDs/passwords safe;
no test certificate is installed in the Windows trust store by this build process.

LM Studio summaries, cross-app content sharing and the TablesCraft rename are deferred.
No model or cloud account is configured by this release.

The local fork also repairs a filled checkbox whose existing on-state appearance is
empty, missing a usable bounding box or absent. Valid custom artwork is preserved;
the repair uses bounded decoding and fresh appearance objects. This was found by an
independent saved-file check, not by relying on the application's own form value view.
