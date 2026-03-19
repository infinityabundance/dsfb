use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use anyhow::{Context, Result};
use printpdf::{BuiltinFont, Mm, PdfDocument};

pub fn write_text_pdf(path: &Path, title: &str, lines: &[String]) -> Result<()> {
    let (document, page1, layer1) = PdfDocument::new(title, Mm(210.0), Mm(297.0), "Layer 1");
    let font = document.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_size = 8.5;
    let left_margin_mm = 14.0;
    let top_margin_mm = 282.0;
    let bottom_margin_mm = 16.0;
    let line_step_mm = 5.1;
    let max_chars = 92;

    let mut current_layer = document.get_page(page1).get_layer(layer1);
    let mut y = top_margin_mm;
    let mut page_index = 1usize;
    for source_line in lines {
        for line in wrap_text(source_line, max_chars) {
            if y < bottom_margin_mm {
                page_index += 1;
                let layer_name = format!("Layer {page_index}");
                let (page, layer) = document.add_page(Mm(210.0), Mm(297.0), layer_name);
                current_layer = document.get_page(page).get_layer(layer);
                y = top_margin_mm;
            }
            current_layer.use_text(line, font_size, Mm(left_margin_mm), Mm(y), &font);
            y -= line_step_mm;
        }
    }

    document
        .save(&mut BufWriter::new(File::create(path).with_context(
            || format!("failed to create {}", path.display()),
        )?))
        .with_context(|| format!("failed to save {}", path.display()))
}

fn wrap_text(line: &str, max_chars: usize) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }
    let mut wrapped = Vec::new();
    let mut current = String::new();
    for token in line.split_whitespace() {
        let next_len = if current.is_empty() {
            token.len()
        } else {
            current.len() + 1 + token.len()
        };
        if next_len > max_chars && !current.is_empty() {
            wrapped.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(token);
    }
    if !current.is_empty() {
        wrapped.push(current);
    }
    if wrapped.is_empty() {
        wrapped.push(String::new());
    }
    wrapped
}
