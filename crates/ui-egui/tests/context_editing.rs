use egui_kittest::{Harness, kittest::Queryable};
use pdfcraft_ui_egui::PdfCraftApp;
fn app() -> PdfCraftApp {
    app_with_permissions(false)
}
fn app_with_permissions(restricted: bool) -> PdfCraftApp {
    let session = pdfcraft_engine::Session::new();
    let (_, bytes) =
        session.convert_to_pdf("Synthetic.txt", &std::sync::Arc::new(b"Select this synthetic sentence for highlighting.".to_vec())).unwrap();
    let mut a = PdfCraftApp::new();
    a.run_inline = true;
    a.system_clipboard = false;
    let bytes = if restricted {
        let mut doc = pdfcraft_cos::Document::open(bytes).unwrap();
        doc.set_encryption(&pdfcraft_cos::NewEncryption {
            algorithm: pdfcraft_cos::Algorithm::Aes256,
            user_password: "",
            owner_password: "synthetic-owner",
            permissions: -1 ^ 0b1_0000,
            encrypt_metadata: true,
            seed: [9; 32],
        })
        .unwrap();
        pdfcraft_cos::write_full(&doc, &Default::default()).unwrap()
    } else {
        (*bytes).clone()
    };
    a.open_bytes("Synthetic.pdf", None, bytes).unwrap();
    a
}

#[test]
fn restricted_pdf_inline_editor_cannot_cut_extracted_text() {
    let mut h = Harness::builder().with_size(egui::vec2(1440.0, 1000.0)).build_eframe(|_| app_with_permissions(true));
    h.run_steps(12);
    let (_, id) = h.state().active_ids().unwrap();
    assert!(!h.state().session.get(id).unwrap().allows_copying());
    assert!(h.state_mut().execute("edit.edit_text"));
    h.run_steps(3);
    let doc = h.state().session.get(id).unwrap();
    let bounds = doc.text_blocks(0).first().unwrap().rect;
    let info = &doc.info.pages[0];
    let page = h.state().views[0].page_screen_rect(0).unwrap();
    let at = egui::pos2(
        page.left() + ((bounds[0] + bounds[2]) * 0.5 / info.width as f64) as f32 * page.width(),
        page.top() + (1.0 - ((bounds[1] + bounds[3]) * 0.5 / info.height as f64)) as f32 * page.height(),
    );
    h.hover_at(at);
    h.drag_at(at);
    h.drop_at(at);
    h.run_steps(4);
    let before = h.state().views[0].line_editor.as_ref().unwrap().text.clone();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run_steps(2);
    h.event(egui::Event::Cut);
    h.run_steps(3);
    assert_eq!(h.state().views[0].line_editor.as_ref().unwrap().text, before);
}
#[test]
fn right_click_selected_text_highlights_and_can_undo() {
    let mut h = Harness::builder().with_size(egui::vec2(1440.0, 1000.0)).wgpu().build_eframe(|_| app());
    h.run_steps(12);
    let (_, id) = h.state().active_ids().unwrap();
    let index = h.state().views.iter().position(|v| v.id == id).unwrap();
    for _ in 0..80 {
        h.state_mut().views[index].select_text(0, 0, 6);
        h.run_steps(1);
        if h.state().views[index].glyph_screen_pos(0, 2).is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let at = h.state().views[index].glyph_screen_pos(0, 2).unwrap();
    h.event(egui::Event::PointerMoved(at));
    for pressed in [true, false] {
        h.event(egui::Event::PointerButton { pos: at, button: egui::PointerButton::Secondary, pressed, modifiers: Default::default() });
        h.run_steps(2);
    }
    h.get_by_label_contains("Mark selected text").hover();
    h.run_steps(5);
    if let Ok(path) = std::env::var("CRAFT_AI_APP_IMAGE") {
        h.render().unwrap().save(path).unwrap();
    }
    h.query_all_by_label_contains("Highlight").last().unwrap().click();
    h.run_steps(4);
    assert!(h.state().session.get(id).unwrap().dirty);
    let annotations = &h.state().session.get(id).unwrap().info.annotations;
    assert!(annotations.iter().any(|a| a.subtype == "Highlight"));
    h.state_mut().session.undo(id).unwrap();
    assert!(!h.state().session.get(id).unwrap().info.annotations.iter().any(|a| a.subtype == "Highlight"));
}
