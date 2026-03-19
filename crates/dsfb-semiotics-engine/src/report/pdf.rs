use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use anyhow::{Context, Result};
use printpdf::image_crate::io::Reader as ImageReader;
use printpdf::{
    BuiltinFont, Image, ImageTransform, IndirectFontRef, Mm, PdfDocument, PdfDocumentReference,
    PdfLayerReference,
};

use crate::engine::types::{FigureArtifact, ReportManifest};

const PORTRAIT_W_MM: f64 = 210.0;
const PORTRAIT_H_MM: f64 = 297.0;
const LANDSCAPE_W_MM: f64 = 297.0;
const LANDSCAPE_H_MM: f64 = 210.0;

const TEXT_LEFT_MM: f64 = 14.0;
const TEXT_TOP_MM: f64 = 282.0;
const TEXT_BOTTOM_MM: f64 = 16.0;
const BODY_LINE_STEP_MM: f64 = 5.1;
const CODE_LINE_STEP_MM: f64 = 3.7;

#[derive(Clone, Debug)]
pub struct PdfTextArtifact {
    pub title: String,
    pub artifact_path: String,
    pub artifact_kind: String,
    pub content: String,
}

pub fn write_artifact_pdf(
    path: &Path,
    title: &str,
    report_lines: &[String],
    figures: &[FigureArtifact],
    manifest: &ReportManifest,
    text_artifacts: &[PdfTextArtifact],
) -> Result<()> {
    let (document, page1, layer1) =
        PdfDocument::new(title, mm(PORTRAIT_W_MM), mm(PORTRAIT_H_MM), "Layer 1");
    let body_font = document.add_builtin_font(BuiltinFont::Helvetica)?;
    let heading_font = document.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let code_font = document.add_builtin_font(BuiltinFont::Courier)?;

    let mut writer = PdfWriter {
        document,
        body_font,
        heading_font,
        code_font,
        page_counter: 1,
        current_layer: None,
        current_page_width_mm: PORTRAIT_W_MM,
        current_page_height_mm: PORTRAIT_H_MM,
    };
    writer.current_layer = Some(writer.document.get_page(page1).get_layer(layer1));

    writer.render_markdown_report(report_lines)?;
    writer.render_figure_pages(figures)?;
    writer.render_artifact_inventory(manifest)?;
    writer.render_text_artifacts(text_artifacts)?;

    writer
        .document
        .save(&mut BufWriter::new(File::create(path).with_context(
            || format!("failed to create {}", path.display()),
        )?))
        .with_context(|| format!("failed to save {}", path.display()))
}

struct PdfWriter {
    document: PdfDocumentReference,
    body_font: IndirectFontRef,
    heading_font: IndirectFontRef,
    code_font: IndirectFontRef,
    page_counter: usize,
    current_layer: Option<PdfLayerReference>,
    current_page_width_mm: f64,
    current_page_height_mm: f64,
}

impl PdfWriter {
    fn render_markdown_report(&mut self, lines: &[String]) -> Result<()> {
        let mut layer = self
            .current_layer
            .clone()
            .context("missing initial PDF layer")?;
        let mut y = TEXT_TOP_MM;

        for source_line in lines {
            let rendered = render_markdown_line(source_line);
            if rendered.is_blank {
                y -= rendered.line_step_mm;
                continue;
            }
            for line in wrap_for_mode(&rendered.text, rendered.max_chars, rendered.code_like) {
                if y < TEXT_BOTTOM_MM {
                    layer = self.add_page(PORTRAIT_W_MM, PORTRAIT_H_MM)?;
                    y = TEXT_TOP_MM;
                }
                layer.use_text(
                    line,
                    rendered.font_size as f32,
                    mm(rendered.x_mm),
                    mm(y),
                    rendered.font(self),
                );
                y -= rendered.line_step_mm;
            }
        }
        self.current_layer = Some(layer);
        Ok(())
    }

    fn render_figure_pages(&mut self, figures: &[FigureArtifact]) -> Result<()> {
        for figure in figures {
            let layer = self.add_page(LANDSCAPE_W_MM, LANDSCAPE_H_MM)?;
            layer.use_text(
                figure.figure_id.clone(),
                18.0,
                mm(14.0),
                mm(196.0),
                &self.heading_font,
            );

            let caption_lines = wrap_for_mode(&figure.caption, 128, false);
            let mut caption_y = 187.0;
            for line in caption_lines {
                layer.use_text(line, 9.5, mm(14.0), mm(caption_y), &self.body_font);
                caption_y -= 5.2;
            }

            let image = ImageReader::open(&figure.png_path)
                .with_context(|| format!("failed to open {}", figure.png_path))?
                .decode()
                .with_context(|| format!("failed to decode {}", figure.png_path))?;
            let image_width_px = image.width() as f64;
            let image_height_px = image.height() as f64;
            let natural_width_mm = image_width_px * 25.4 / 300.0;
            let natural_height_mm = image_height_px * 25.4 / 300.0;
            let max_width_mm = LANDSCAPE_W_MM - 28.0;
            let max_height_mm = 150.0;
            let scale = (max_width_mm / natural_width_mm).min(max_height_mm / natural_height_mm);
            let scale = if scale.is_finite() && scale > 0.0 {
                scale as f32
            } else {
                1.0
            };
            let rendered_width_mm = natural_width_mm * scale as f64;
            let image_x = (LANDSCAPE_W_MM - rendered_width_mm) / 2.0;
            let image_y = 26.0;

            Image::from_dynamic_image(&image).add_to_layer(
                layer.clone(),
                ImageTransform {
                    translate_x: Some(mm(image_x)),
                    translate_y: Some(mm(image_y)),
                    scale_x: Some(scale),
                    scale_y: Some(scale),
                    dpi: Some(300.0),
                    ..Default::default()
                },
            );

            let footer_lines = [
                format!("PNG artifact: {}", figure.png_path),
                format!("SVG artifact: {}", figure.svg_path),
            ];
            let mut footer_y = 16.0;
            for line in footer_lines {
                for wrapped in wrap_for_mode(&line, 160, true) {
                    layer.use_text(wrapped, 7.2, mm(14.0), mm(footer_y), &self.code_font);
                    footer_y -= 4.0;
                }
            }
        }
        Ok(())
    }

    fn render_artifact_inventory(&mut self, manifest: &ReportManifest) -> Result<()> {
        let mut lines = vec![
            "# Artifact Inventory".to_string(),
            String::new(),
            format!("Run directory: {}", manifest.run_dir),
            format!("Report markdown: {}", manifest.report_markdown),
            format!("Report pdf: {}", manifest.report_pdf),
            format!("Zip archive: {}", manifest.zip_archive),
            String::new(),
            format!("Figure paths: {}", manifest.figure_paths.len()),
        ];
        lines.extend(
            manifest
                .figure_paths
                .iter()
                .map(|path| format!("- {}", path)),
        );
        lines.push(String::new());
        lines.push(format!("CSV paths: {}", manifest.csv_paths.len()));
        lines.extend(manifest.csv_paths.iter().map(|path| format!("- {}", path)));
        lines.push(String::new());
        lines.push(format!("JSON paths: {}", manifest.json_paths.len()));
        lines.extend(manifest.json_paths.iter().map(|path| format!("- {}", path)));

        self.render_markdown_report(&lines)
    }

    fn render_text_artifacts(&mut self, text_artifacts: &[PdfTextArtifact]) -> Result<()> {
        for artifact in text_artifacts {
            let header_lines = vec![
                format!("# {}", artifact.title),
                String::new(),
                format!("kind: {}", artifact.artifact_kind),
                format!("path: {}", artifact.artifact_path),
                String::new(),
            ];
            self.render_markdown_report(&header_lines)?;
            self.render_code_block(&artifact.content)?;
        }
        Ok(())
    }

    fn render_code_block(&mut self, content: &str) -> Result<()> {
        let mut layer = self.add_page(PORTRAIT_W_MM, PORTRAIT_H_MM)?;
        let mut y = TEXT_TOP_MM;

        for source_line in content.lines() {
            let wrapped = wrap_for_mode(source_line, 122, true);
            let wrapped = if wrapped.is_empty() {
                vec![String::new()]
            } else {
                wrapped
            };
            for line in wrapped {
                if y < TEXT_BOTTOM_MM {
                    layer = self.add_page(PORTRAIT_W_MM, PORTRAIT_H_MM)?;
                    y = TEXT_TOP_MM;
                }
                layer.use_text(line, 6.8, mm(TEXT_LEFT_MM), mm(y), &self.code_font);
                y -= CODE_LINE_STEP_MM;
            }
        }
        self.current_layer = Some(layer);
        Ok(())
    }

    fn add_page(&mut self, width_mm: f64, height_mm: f64) -> Result<PdfLayerReference> {
        self.page_counter += 1;
        self.current_page_width_mm = width_mm;
        self.current_page_height_mm = height_mm;
        let (page, layer) = self.document.add_page(
            mm(width_mm),
            mm(height_mm),
            format!("Layer {}", self.page_counter),
        );
        Ok(self.document.get_page(page).get_layer(layer))
    }
}

struct RenderedLine {
    text: String,
    font_size: f64,
    x_mm: f64,
    line_step_mm: f64,
    max_chars: usize,
    code_like: bool,
    is_blank: bool,
    font_kind: FontKind,
}

impl RenderedLine {
    fn font<'a>(&self, writer: &'a PdfWriter) -> &'a IndirectFontRef {
        match self.font_kind {
            FontKind::Body => &writer.body_font,
            FontKind::Heading => &writer.heading_font,
        }
    }
}

enum FontKind {
    Body,
    Heading,
}

fn render_markdown_line(source: &str) -> RenderedLine {
    if source.trim().is_empty() {
        return RenderedLine {
            text: String::new(),
            font_size: 8.5,
            x_mm: TEXT_LEFT_MM,
            line_step_mm: BODY_LINE_STEP_MM,
            max_chars: 92,
            code_like: false,
            is_blank: true,
            font_kind: FontKind::Body,
        };
    }

    if let Some(text) = source.strip_prefix("# ") {
        return RenderedLine {
            text: text.to_string(),
            font_size: 16.0,
            x_mm: TEXT_LEFT_MM,
            line_step_mm: 8.0,
            max_chars: 70,
            code_like: false,
            is_blank: false,
            font_kind: FontKind::Heading,
        };
    }
    if let Some(text) = source.strip_prefix("## ") {
        return RenderedLine {
            text: text.to_string(),
            font_size: 12.5,
            x_mm: TEXT_LEFT_MM,
            line_step_mm: 6.6,
            max_chars: 82,
            code_like: false,
            is_blank: false,
            font_kind: FontKind::Heading,
        };
    }
    if let Some(text) = source.strip_prefix("### ") {
        return RenderedLine {
            text: text.to_string(),
            font_size: 10.6,
            x_mm: TEXT_LEFT_MM,
            line_step_mm: 5.8,
            max_chars: 88,
            code_like: false,
            is_blank: false,
            font_kind: FontKind::Heading,
        };
    }
    if let Some(text) = source.strip_prefix("- ") {
        return RenderedLine {
            text: format!("• {}", text),
            font_size: 8.5,
            x_mm: TEXT_LEFT_MM + 3.0,
            line_step_mm: BODY_LINE_STEP_MM,
            max_chars: 88,
            code_like: false,
            is_blank: false,
            font_kind: FontKind::Body,
        };
    }

    RenderedLine {
        text: source.to_string(),
        font_size: 8.5,
        x_mm: TEXT_LEFT_MM,
        line_step_mm: BODY_LINE_STEP_MM,
        max_chars: 92,
        code_like: false,
        is_blank: false,
        font_kind: FontKind::Body,
    }
}

fn wrap_for_mode(line: &str, max_chars: usize, code_like: bool) -> Vec<String> {
    if code_like {
        wrap_fixed_width(line, max_chars)
    } else {
        wrap_text(line, max_chars)
    }
}

fn wrap_text(line: &str, max_chars: usize) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }

    let mut wrapped = Vec::new();
    let mut current = String::new();
    for token in line.split_whitespace() {
        if token.len() > max_chars {
            if !current.is_empty() {
                wrapped.push(std::mem::take(&mut current));
            }
            wrapped.extend(wrap_fixed_width(token, max_chars));
            continue;
        }
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

fn wrap_fixed_width(line: &str, max_chars: usize) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }

    let chars = line.chars().collect::<Vec<_>>();
    chars
        .chunks(max_chars.max(1))
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}

fn mm(value: f64) -> Mm {
    Mm(value as f32)
}
