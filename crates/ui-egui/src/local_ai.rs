//! Permission-checked extraction of the currently displayed PDF snapshot.
use crate::PdfCraftApp;
use craft_local_ai::Source;

#[derive(Clone)]
struct SelectionRequest {
    key: String,
    source: Result<Source, String>,
}
fn document_key(doc: &pdfcraft_engine::Document, view: &crate::canvas::DocView) -> String {
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    if let Some(editor) = &view.line_editor {
        editor.text.hash(&mut hash);
    }
    format!("{}:{}:{}", view.id.0, doc.edit_generation(), hash.finish())
}
pub(crate) fn selection_menu(ui: &mut egui::Ui, doc: &pdfcraft_engine::Document, view: &crate::canvas::DocView) {
    if ui.button("Ask local AI").clicked() {
        let request = SelectionRequest { key: document_key(doc, view), source: selection_source(doc, view) };
        ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("pdf-ai-selection"), request));
        ui.close();
    }
}
pub(crate) fn image_menu(ui: &mut egui::Ui, view: &crate::canvas::DocView) {
    if ui.button("Ask local AI").clicked() {
        // No image bytes, OCR, page text or neighbouring text are read implicitly.
        ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("pdf-ai-image"), view.id));
        ui.close();
    }
}
fn normalized(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
fn selection_source(doc: &pdfcraft_engine::Document, view: &crate::canvas::DocView) -> Result<Source, String> {
    doc.check_copying().map_err(|e| e.to_string())?;
    let mut source = Source::new("Selected PDF text / objects; image pixels excluded");
    if view.prepare.selected.is_some() {
        return Err("Select readable text from the form before asking.".into());
    }
    if let Some((page, objects)) = view.objects.ai_selection(doc.edit_generation())? {
        source.partial = objects.len() > 2000;
        let displayed = view.page_text(page).map(|text| normalized(&text.plain_text()));
        for object in objects.iter().take(2000) {
            if let Some(text) = &object.text {
                // Native object inventory can contain hidden-layer text. Only forward text
                // present in the displayed page, or explicitly added text. Never widen scope.
                if object.target.kind != pdfcraft_engine::ObjectKind::Added
                    && !displayed.as_ref().is_some_and(|page| !normalized(text).is_empty() && page.contains(&normalized(text)))
                {
                    return Err("This object has no verified displayed text. Select its visible text instead.".into());
                }
                if !source.push(&format!("Page {} / selected {} {}", page + 1, object.content_type, object.target.index + 1), text) {
                    break;
                }
            }
        }
        return Ok(source);
    }
    if view.image_selection.is_some() || view.picked_image.is_some() {
        return Err("This image has no readable text. Run OCR and select its text first.".into());
    }
    if let Some(editor) = &view.line_editor {
        let text = match &editor.selection {
            Some(range) => editor.text.get(range.clone()).ok_or("The text selection changed. Select it again.")?,
            None => &editor.text,
        };
        source.push(&format!("Page {} / selected text object", editor.page + 1), text);
        return Ok(source);
    }
    if let Some((page, index)) = view.content.selected {
        if let Some(added) = doc.added().get(index)
            && added.page == page
            && let pdfcraft_engine::AddedContent::Text(text) = &added.content
        {
            source.push(&format!("Page {} / selected added text", page + 1), &text.text);
        }
        return Ok(source);
    }
    let text = view.selected_text().ok_or("Select readable text or a text object first.")?;
    let page = view.selection_quads(&doc.info).map(|(page, _)| page).unwrap_or(view.current);
    source.push(&format!("Page {} / text selection", page + 1), &text);
    Ok(source)
}
pub fn show(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let key = app.active_ids().and_then(|(view, id)| app.session.get(id).map(|doc| document_key(doc, &app.views[view])));
    if let Some(request) = ui.ctx().data_mut(|d| {
        let id = egui::Id::new("pdf-ai-selection");
        let request = d.get_temp::<SelectionRequest>(id);
        d.remove::<SelectionRequest>(id);
        request
    }) {
        if key.as_deref() == Some(&request.key) {
            app.local_ai.open_selection(ui.ctx(), key.clone(), move |_| request.source);
        }
    }
    if let Some(id) = ui.ctx().data_mut(|d| {
        let id = egui::Id::new("pdf-ai-image");
        let request = d.get_temp::<pdfcraft_engine::DocId>(id);
        d.remove::<pdfcraft_engine::DocId>(id);
        request
    }) {
        if app.active_ids().is_some_and(|(_, active)| active == id) {
            let result = app.session.get(id).ok_or("PDF closed.").and_then(|doc| doc.check_copying().map_err(|_| "Copying is restricted."));
            app.local_ai.open_selection(ui.ctx(), key.clone(), move |_| {
                result?;
                Err("This image has no readable text. Run OCR and select its text first.".into())
            });
        }
    }
    if let Some(scope) = app.local_ai.show(ui, key, &["Selected text / objects", "This page", "Selected pages", "Document (128 pages max)"]) {
        if scope == 0 {
            let source = app
                .active_ids()
                .ok_or("Open a PDF first.".to_string())
                .and_then(|(view, id)| selection_source(app.session.get(id).ok_or("PDF closed.")?, &app.views[view]));
            app.local_ai.prepare(ui.ctx(), move |_| source);
            return;
        }
        let prepared: Result<_, String> = (|| {
            let (view, id) = app.active_ids().ok_or("Open a PDF first.")?;
            let doc = app.session.get(id).ok_or("The PDF is no longer open.")?;
            doc.check_copying().map_err(|e| e.to_string())?;
            let current = app.views[view].current;
            let selected = app.views[view].selected_text();
            let selection_page = app.views[view].selection_quads(&doc.info).map(|(page, _)| page).unwrap_or(current);
            let pages: Vec<_> = match scope {
                0 => Vec::new(),
                1 => vec![current],
                2 => app.views[view].selected.iter().copied().collect(),
                _ => (0..doc.info.pages.len().min(128)).collect(),
            };
            let partial = scope == 3 && doc.info.pages.len() > 128;
            if scope == 2 && (pages.is_empty() || pages.len() > 128) {
                return Err("Select 1–128 pages in the Pages panel first.".into());
            }
            Ok((doc.export_source(), selected, selection_page, pages, partial))
        })();
        app.local_ai.prepare(ui.ctx(), move |cancel| {
            let (snapshot, selected, selection_page, pages, partial) = prepared?;
            let mut source = Source::new(match scope {
                0 => "Selected PDF text snapshot",
                1 => "Current PDF page snapshot",
                2 => "Selected PDF pages snapshot",
                _ => "PDF text snapshot",
            });
            source.partial = partial;
            if scope == 0 {
                source.push(
                    &format!("Page {} / text selection", selection_page + 1),
                    &selected.ok_or("Select readable PDF text first; scanned images need OCR.")?,
                );
            } else {
                pdfcraft_engine::guard(|| {
                    let mut exporter = pdfcraft_engine::export::Exporter::from_source(snapshot);
                    for page in pages {
                        if cancel.is_cancelled() {
                            return Err("Preview cancelled.".into());
                        }
                        if !source.push(&format!("Page {}", page + 1), &exporter.text(page)?) {
                            break;
                        }
                    }
                    Ok::<_, String>(())
                })??;
            }
            Ok(source)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};
    fn document(restricted: bool) -> Vec<u8> {
        let session = pdfcraft_engine::Session::new();
        let (_, bytes) =
            session.convert_to_pdf("Synthetic.txt", &std::sync::Arc::new(b"The project deadline is 31 December 2026.".to_vec())).unwrap();
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
}
