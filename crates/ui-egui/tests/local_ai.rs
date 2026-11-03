// Focused real-window extraction and copy-permission regression tests.

use egui_kittest::{Harness, kittest::Queryable};
use pdfcraft_ui_egui::PdfCraftApp;
fn document(restricted: bool) -> Vec<u8> {
    let session = pdfcraft_engine::Session::new();
    let (_, bytes) = session.convert_to_pdf("Synthetic.txt", &std::sync::Arc::new(b"The project deadline is 31 December 2026.".to_vec())).unwrap();
    if !restricted {
        return (*bytes).clone();
    }
    let mut doc = pdfcraft_cos::Document::open(bytes).unwrap();
    doc.set_encryption(&pdfcraft_cos::NewEncryption {
        algorithm: pdfcraft_cos::Algorithm::Aes256,
        user_password: "",
        owner_password: "synthetic-owner",
        permissions: -1 ^ 0b1_0000,
        encrypt_metadata: true,
        seed: [7; 32],
    })
    .unwrap();
    pdfcraft_cos::write_full(&doc, &Default::default()).unwrap()
}
fn run(restricted: bool) {
    let bytes = document(restricted);
    let mut h = Harness::builder().with_size(egui::vec2(1440.0, 1000.0)).build_eframe(move |_| {
        let mut app = PdfCraftApp::new();
        app.run_inline = true;
        app.open_bytes("Synthetic.pdf", None, bytes).unwrap();
        app.local_ai.open = true;
        app.local_ai.scope = 1;
        app
    });
    h.run_steps(8);
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Preview").click();
    for _ in 0..100 {
        h.run_steps(1);
        if !h.state().local_ai.busy() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    if restricted {
        assert!(h.state().local_ai.preview().is_none());
        assert!(h.state().local_ai.notice().unwrap_or_default().contains("copying"));
    } else {
        assert!(
            h.state()
                .local_ai
                .preview()
                .expect("page preview")
                .excerpts
                .iter()
                .any(|e| e.location == "Page 1" && e.text.contains("31 December 2026"))
        );
        if let Ok(path) = std::env::var("CRAFT_AI_APP_IMAGE") {
            h.render().unwrap().save(path).unwrap();
        }
    }
    let (_, id) = h.state().active_ids().unwrap();
    assert!(!h.state().session.get(id).unwrap().dirty);
}
#[test]
fn local_ai_pdf_preview_uses_current_text_without_editing() {
    run(false);
}
#[test]
fn local_ai_pdf_copy_restriction_is_enforced_before_extraction() {
    run(true);
}

#[test]
fn local_ai_context_menu_captures_text_and_preserves_the_document() {
    let bytes = document(false);
    let mut h = Harness::builder().with_size(egui::vec2(1440., 1000.)).build_eframe(move |_| {
        let mut app = PdfCraftApp::new();
        app.run_inline = true;
        app.open_bytes("Synthetic.pdf", None, bytes).unwrap();
        app
    });
    h.run_steps(12);
    h.state_mut().views[0].select_text(0, 0, 2);
    h.run_steps(2);
    let selected = h.state().views[0].selected_text().expect("rendered selection");
    let pos = h.state().views[0].page_screen_rect(0).unwrap().center();
    h.event(egui::Event::PointerMoved(pos));
    h.event(egui::Event::PointerButton { pos, button: egui::PointerButton::Secondary, pressed: true, modifiers: Default::default() });
    h.event(egui::Event::PointerButton { pos, button: egui::PointerButton::Secondary, pressed: false, modifiers: Default::default() });
    h.run_steps(3);
    let buttons: Vec<_> = h.get_all_by_role_and_label(egui::accesskit::Role::Button, "Ask local AI").collect();
    assert_eq!(buttons.len(), 2, "toolbar and page context menu");
    buttons.last().unwrap().click();
    h.run_steps(3);
    h.state_mut().views[0].select_text(0, 5, 10);
    for _ in 0..100 {
        h.run_steps(1);
        if h.state().local_ai.preview().is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let plan = h.state().local_ai.preview().unwrap();
    assert_eq!(plan.excerpts.len(), 1);
    assert_eq!(plan.excerpts[0].text, selected.trim());
    let (_, id) = h.state().active_ids().unwrap();
    assert!(!h.state().session.get(id).unwrap().dirty);
}
