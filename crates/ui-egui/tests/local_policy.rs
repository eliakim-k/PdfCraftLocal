#![cfg(feature = "local-only")]

use pdfcraft_ui_egui::PdfCraftApp;

#[test]
fn local_listener_and_external_dispatch_are_blocked() {
    let mut app = PdfCraftApp::new();
    let ctx = egui::Context::default();
    let client = app.attach_control(&ctx);
    assert!(pdfcraft_ui_egui::control::serve(client).is_err());
    app.open_url("https://example.invalid/selected-content");
    assert!(app.last_opened_url.is_none());
    assert!(ctx.output(|o| o.commands.is_empty()));
    app.session.set_javascript(true);
    assert!(!app.session.javascript());
}
