//  Copyright (C) 2026 wellman4
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License.

use crate::backend::Backend;
use crate::lexer::{AbstractKind, ConclusionKind, ContentsKind, StructuralKind};
use crate::parser::ast::{Document, ListContainer, Node};
use printpdf::*;
use swash::FontRef;
use swash::shape::ShapeContext;

#[allow(dead_code)]
pub struct LayoutConfig {
    pub page_width: Mm,
    pub page_height: Mm,
    pub base_font_size: Pt,
    pub small_font_size: Pt,
    pub margin_left: Mm,
    pub margin_top: Mm,
    pub margin_bottom: Mm,
    pub margin_right: Mm,
    pub indent: Mm,
}

#[allow(dead_code)]
impl LayoutConfig {
    pub fn from_node(doc: &Document) -> Self {
        let get_num = |key: &str, default: f32| -> f32 {
            doc.attributes
                .get(key)
                .and_then(|s| s.parse().ok())
                .unwrap_or(default)
        };

        Self {
            page_width: Mm(210.0),
            page_height: Mm(297.0),
            base_font_size: Pt(get_num("основной_размер_шрифта", 12.0)),
            small_font_size: Pt(get_num("малый_размер_шрифта", 10.0)),
            margin_left: Mm(get_num("левое_поле", 25.0)),
            margin_top: Mm(get_num("верхнее_поле", 20.0)),
            margin_bottom: Mm(get_num("нижнее_поле", 20.0)),
            margin_right: Mm(get_num("правое_поле", 10.0)),
            indent: Mm(12.5),
        }
    }
}

struct RenderState {
    pages: Vec<PdfPage>,
    current_ops: Vec<Op>,
    current_y: Mm,
    page_count: usize,
    show_page_numbers: bool,
    current_list_prefix: ListPrefix,
}

impl RenderState {
    fn new(l_cfg: &LayoutConfig) -> Self {
        Self {
            pages: Vec::new(),
            current_ops: Vec::new(),
            current_y: l_cfg.page_height - l_cfg.margin_top,
            page_count: 0,
            show_page_numbers: false,
            current_list_prefix: ListPrefix::Auto,
        }
    }
}

pub struct PdfFont {
    pub handle: PdfFontHandle,
    pub data: &'static [u8],
}

impl PdfFont {
    pub fn as_swash(&self) -> FontRef<'_> {
        FontRef::from_index(self.data, 0).expect("Failed to create FontRef")
    }
}

#[allow(dead_code)]
pub struct FontFamily {
    pub regular: PdfFont,
    pub bold: PdfFont,
    pub italic: PdfFont,
}

#[derive(PartialEq)]
#[allow(dead_code)]
pub enum TextAlign {
    Left,
    Center,
    Justify,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ListPrefix {
    None,
    Auto,
    Number,
}

fn measure_width(text: &str, pdf_font: &PdfFont, font_size: f32) -> f32 {
    let font_ref = pdf_font.as_swash();

    let mut context = ShapeContext::new();

    let mut shaper = context.builder(font_ref).size(font_size).build();

    shaper.add_str(text);

    let mut total_width = 0.0;
    shaper.shape_with(|cluster| {
        for glyph in cluster.glyphs {
            total_width += glyph.advance;
        }
    });

    total_width
}

fn get_gost_char(idx: usize) -> String {
    let chars = [
        'а', 'б', 'в', 'г', 'д', 'е', 'ж', 'и', 'к', 'л', 'м', 'н', 'п', 'р', 'с', 'т', 'у', 'ф',
        'х', 'ц', 'ш', 'щ', 'э', 'ю', 'я',
    ];
    let c = chars.get(idx).unwrap_or(&'?');
    format!("{})", c)
}

pub struct PdfBackend;

impl PdfBackend {
    pub fn new() -> Self {
        Self
    }

    fn load_fonts(&self, doc: &mut PdfDocument) -> Result<FontFamily, Box<dyn std::error::Error>> {
        let regular_data =
            include_bytes!("../../assets/fonts/PT-Astra-Serif/pt-astra-serif_regular.ttf");
        let bold_data = include_bytes!("../../assets/fonts/PT-Astra-Serif/pt-astra-serif_bold.ttf");
        let italic_data =
            include_bytes!("../../assets/fonts/PT-Astra-Serif/pt-astra-serif_italic.ttf");

        let mut load_style =
            |data: &'static [u8], name: &str| -> Result<PdfFont, Box<dyn std::error::Error>> {
                let parsed = ParsedFont::from_bytes(data, 0, &mut Vec::new())
                    .ok_or(format!("Ошибка парсинга шрифта {}", name))?;

                let handle = PdfFontHandle::External(doc.add_font(&parsed));

                Ok(PdfFont { handle, data })
            };

        Ok(FontFamily {
            regular: load_style(regular_data, "regular")?,
            bold: load_style(bold_data, "bold")?,
            italic: load_style(italic_data, "italic")?,
        })
    }

    fn setup_new_page(&self, state: &mut RenderState, l_cfg: &LayoutConfig, font: &PdfFont) {
        if !state.current_ops.is_empty() {
            state.current_ops.push(Op::EndTextSection);

            if state.show_page_numbers {
                self.draw_page_number(state.page_count, font, l_cfg, &mut state.current_ops);
            }

            let ops = std::mem::take(&mut state.current_ops);
            state
                .pages
                .push(PdfPage::new(l_cfg.page_width, l_cfg.page_height, ops));
        }

        state.page_count += 1;

        state.current_ops.push(Op::SaveGraphicsState);
        self.draw_margins(l_cfg, &mut state.current_ops);
        state.current_ops.push(Op::RestoreGraphicsState);

        state.current_ops.push(Op::StartTextSection);

        state.current_ops.push(Op::SetFillColor {
            col: Color::Greyscale(Greyscale::new(0.0, None)),
        });
        state.current_ops.push(Op::SetTextRenderingMode {
            mode: TextRenderingMode::Fill,
        });

        state.current_ops.push(Op::SetFont {
            font: font.handle.clone(),
            size: l_cfg.base_font_size,
        });

        let font_size_mm = l_cfg.base_font_size.0 * (25.4 / 72.0);
        state.current_y = l_cfg.page_height - l_cfg.margin_top - Mm(font_size_mm);

        state.current_ops.push(Op::SetTextCursor {
            pos: Point::new(l_cfg.margin_left, state.current_y),
        });
    }

    fn write_paragraph(
        &self,
        text: &str,
        font: &PdfFont,
        font_size: Pt,
        multiplier: f32,
        left_padding: Mm,
        first_line_indent: Mm,
        align: TextAlign,
        state: &mut RenderState,
        l_cfg: &LayoutConfig,
    ) {
        if text.is_empty() {
            return;
        }
        let mm_to_pt = 72.0 / 25.4;
        let lh_mm = (font_size.0 * multiplier) / mm_to_pt;

        let content_max_w_mm =
            l_cfg.page_width.0 - l_cfg.margin_left.0 - l_cfg.margin_right.0 - left_padding.0;

        let first_line_limit_mm = content_max_w_mm - first_line_indent.0;

        let lines = self.break_lines(
            text,
            content_max_w_mm,
            first_line_limit_mm,
            font_size.0,
            font,
        );
        let lines_count = lines.len();

        state.current_ops.push(Op::SetFont {
            font: font.handle.clone(),
            size: font_size,
        });
        state.current_ops.push(Op::SetLineHeight {
            lh: font_size * multiplier,
        });

        for (i, line) in lines.iter().enumerate() {
            let is_at_bottom = state.current_y.0 - lh_mm < l_cfg.margin_bottom.0;

            if is_at_bottom {
                self.setup_new_page(state, l_cfg, font);
                state.current_ops.push(Op::SetFont {
                    font: font.handle.clone(),
                    size: font_size,
                });
                state.current_ops.push(Op::SetLineHeight {
                    lh: font_size * multiplier,
                });
            }

            let mut word_spacing_pt = 0.0;
            let is_last_line = i == lines_count - 1;

            let x_pt = match align {
                TextAlign::Center => {
                    let text_width_pt = measure_width(line, font, font_size.0);
                    let content_width_pt = content_max_w_mm * mm_to_pt;

                    let offset_pt = (content_width_pt - text_width_pt) / 2.0;
                    (l_cfg.margin_left.0 + left_padding.0) * mm_to_pt + offset_pt
                }
                TextAlign::Left | TextAlign::Justify => {
                    let current_indent = if i == 0 { first_line_indent.0 } else { 0.0 };
                    let base_x_pt =
                        (l_cfg.margin_left.0 + left_padding.0 + current_indent) * mm_to_pt;

                    if align == TextAlign::Justify && !is_last_line {
                        let line = line.trim();
                        let actual_line_width_pt = measure_width(line, font, font_size.0);
                        let current_limit_pt = ((if i == 0 {
                            first_line_limit_mm
                        } else {
                            content_max_w_mm
                        }) * mm_to_pt)
                            - 1.5;

                        let diff_pt = current_limit_pt - actual_line_width_pt;

                        let words_in_line: Vec<&str> = line.split_whitespace().collect();
                        let space_count = words_in_line.len().saturating_sub(1);

                        if space_count > 0 && diff_pt > 0.0 {
                            word_spacing_pt = diff_pt / space_count as f32;
                        }
                    }
                    base_x_pt
                }
            };

            let mut items = Vec::new();
            if word_spacing_pt > 0.0 {
                let words: Vec<&str> = line.split_whitespace().collect();
                for (idx, word) in words.iter().enumerate() {
                    items.push(TextItem::Text(word.to_string()));
                    if idx < words.len() - 1 {
                        let standard_space_width_pt = measure_width(" ", font, font_size.0);
                        let total_gap_pt = standard_space_width_pt + word_spacing_pt;
                        let offset = -(total_gap_pt * 1000.0 / font_size.0);
                        items.push(TextItem::Offset(offset));
                    }
                }
            } else {
                items.push(TextItem::Text(line.clone()));
            }

            state.current_ops.push(Op::SetTextMatrix {
                matrix: TextMatrix::Translate(Pt(x_pt), state.current_y.into()),
            });

            state.current_ops.push(Op::ShowText { items });

            state.current_y.0 -= lh_mm;
        }
    }

    fn write_list(
        &self,
        container: &ListContainer,
        fonts: &FontFamily,
        state: &mut RenderState,
        l_cfg: &LayoutConfig,
        level: u8,
        is_parent_last: bool,
    ) {
        let items = &container.items;
        let total_items = items.len();

        for (idx, item_node) in container.items.iter().enumerate() {
            if let Node::ListItem(li) = item_node {
                let is_last_in_current = idx == total_items - 1;
                let has_inner = li.children.iter().any(|c| matches!(c, Node::List(_)));
                let is_absolutely_last = is_last_in_current && is_parent_last && !has_inner;

                let mut prefix_str = match state.current_list_prefix {
                    ListPrefix::Auto => {
                        if level == 1 {
                            if has_inner {
                                format!("{})", get_gost_char(idx))
                            } else {
                                "-".to_string()
                            }
                        } else {
                            format!("{})", idx + 1)
                        }
                    }
                    ListPrefix::None => "".to_string(),
                    ListPrefix::Number => format!("{}", idx + 1),
                };

                if !prefix_str.is_empty() {
                    prefix_str.push(' ');
                }

                let raw_text = li.text.trim();
                if !raw_text.is_empty() {
                    let final_text = if state.current_list_prefix == ListPrefix::Auto {
                        let target_char = if is_absolutely_last { "." } else { ";" };
                        self.fix_punctuation(&raw_text, target_char)
                    } else {
                        raw_text.to_string()
                    };

                    let left_padding = Mm((level.saturating_sub(1)) as f32 * l_cfg.indent.0);

                    self.write_paragraph(
                        &format!("{}{}", prefix_str, final_text),
                        &fonts.regular,
                        l_cfg.base_font_size,
                        1.5,
                        left_padding,
                        l_cfg.indent,
                        TextAlign::Justify,
                        state,
                        l_cfg,
                    );
                }

                for child in &li.children {
                    match child {
                        Node::List(sub_container) => {
                            self.write_list(
                                sub_container,
                                fonts,
                                state,
                                l_cfg,
                                level + 1,
                                is_last_in_current && is_parent_last,
                            );
                        }
                        other => {
                            self.collect_ops(other, l_cfg, fonts, state);
                        }
                    }
                }
            }
        }
    }

    fn break_lines(
        &self,
        text: &str,
        max_w_mm: f32,
        first_line_max_w_mm: f32,
        font_size: f32,
        font: &PdfFont,
    ) -> Vec<String> {
        let mm_to_pt = 72.0 / 25.4;
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            let max_pt = if lines.is_empty() {
                first_line_max_w_mm * mm_to_pt
            } else {
                max_w_mm * mm_to_pt
            };

            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };

            let current_width = measure_width(&test_line, font, font_size);

            if current_width <= max_pt {
                current_line = test_line;
            } else {
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        lines
    }

    fn collect_ops(
        &self,
        node: &Node,
        l_cfg: &LayoutConfig,
        fonts: &FontFamily,
        state: &mut RenderState,
    ) {
        match node {
            Node::Document(doc) => {
                for child in &doc.children {
                    self.collect_ops(child, l_cfg, fonts, state);
                }
            }

            Node::Structural(el) => {
                if el.kind != StructuralKind::MainPart {
                    self.setup_new_page(state, l_cfg, &fonts.regular);

                    if matches!(el.kind, StructuralKind::Contents(_)) {
                        state.show_page_numbers = true;
                    }

                    let title = match &el.kind {
                        StructuralKind::TitlePage => "ТИТУЛЬНЫЙ ЛИСТ",
                        StructuralKind::NotesPage => "ЛИСТ ДЛЯ ЗАМЕЧАНИЙ",
                        StructuralKind::Abstract(AbstractKind::Extended) => "РЕФЕРАТ",
                        StructuralKind::Abstract(AbstractKind::Short) => "АННОТАЦИЯ",
                        StructuralKind::Contents(ContentsKind::Collection) => "СОДЕРЖАНИЕ",
                        StructuralKind::Contents(ContentsKind::Integrated) => "ОГЛАВЛЕНИЕ",
                        StructuralKind::NormativeReferences => "НОРМАТИВНЫЕ ССЫЛКИ",
                        StructuralKind::Definitions => "ОПРЕДЕЛЕНИЯ, ОБОЗНАЧЕНИЯ И СОКРАЩЕНИЯ",
                        StructuralKind::Introduction => "ВВЕДЕНИЕ",
                        StructuralKind::Conclusion(ConclusionKind::Final) => "ЗАКЛЮЧЕНИЕ",
                        StructuralKind::Conclusion(ConclusionKind::Summary) => "ВЫВОДЫ",
                        StructuralKind::Sources => "СПИСОК ИСПОЛЬЗОВАННЫХ ИСТОЧНИКОВ",
                        StructuralKind::Appendix => "ПРИЛОЖЕНИЕ",
                        StructuralKind::IndependenceStatement => {
                            "СВЕДЕНИЯ О САМОСТОЯТЕЛЬНОСТИ ВЫПОЛНЕНИЯ РАБОТЫ"
                        }
                        StructuralKind::MainPart => "",
                    };

                    self.write_paragraph(
                        title,
                        &fonts.regular,
                        l_cfg.base_font_size,
                        1.5,
                        Mm(0.0),
                        Mm(0.0),
                        TextAlign::Center,
                        state,
                        l_cfg,
                    );
                    state.current_y.0 -= 4.23;

                    state.current_list_prefix = match &el.kind {
                        StructuralKind::NormativeReferences => {
                            self.write_paragraph(
                                "В настоящей текстовом документе использованы ссылки на следующие нормативные документы:",
                                &fonts.regular,
                                l_cfg.base_font_size,
                                1.5,
                                Mm(0.0),
                                Mm(12.5),
                                TextAlign::Justify,
                                state,
                                l_cfg,
                            );
                            ListPrefix::None
                        }
                        StructuralKind::Definitions => {
                            self.write_paragraph(
                                "В настоящем текстовом документе применяются следующие определения, обозначения и сокращения:",
                                &fonts.regular,
                                l_cfg.base_font_size,
                                1.5,
                                Mm(0.0),
                                Mm(12.5),
                                TextAlign::Justify,
                                state,
                                l_cfg,
                            );
                            ListPrefix::None
                        }
                        StructuralKind::Sources => ListPrefix::Number,
                        _ => ListPrefix::Auto,
                    };
                }

                for child in &el.children {
                    self.collect_ops(child, l_cfg, fonts, state);
                }
            }

            Node::List(container) => {
                self.write_list(container, fonts, state, l_cfg, 1, true);
            }

            Node::Paragraph(p) => {
                self.write_paragraph(
                    &p.text,
                    &fonts.regular,
                    l_cfg.base_font_size,
                    1.5,
                    Mm(0.0),
                    Mm(12.5),
                    TextAlign::Justify,
                    state,
                    l_cfg,
                );
            }
            _ => {}
        }
    }

    fn draw_margins(&self, l_cfg: &LayoutConfig, ops: &mut Vec<Op>) {
        let thickness_pt: f32 = 0.5;
        let offset_mm: Mm = Mm::from(Pt(thickness_pt / 2.0));

        let x_min: Mm = l_cfg.margin_left;
        let x_max: Mm = l_cfg.page_width - l_cfg.margin_right;
        let y_min: Mm = l_cfg.margin_bottom;
        let y_max: Mm = l_cfg.page_height - l_cfg.margin_top;

        ops.push(Op::SetOutlineColor {
            col: Color::Rgb(Rgb::new(0.8, 0.8, 0.8, None)),
        });
        ops.push(Op::SetOutlineThickness {
            pt: Pt(thickness_pt),
        });

        let border = Line {
            points: vec![
                LinePoint {
                    p: Point::new(x_min + offset_mm, y_min + offset_mm),
                    bezier: false,
                },
                LinePoint {
                    p: Point::new(x_max - offset_mm, y_min + offset_mm),
                    bezier: false,
                },
                LinePoint {
                    p: Point::new(x_max - offset_mm, y_max - offset_mm),
                    bezier: false,
                },
                LinePoint {
                    p: Point::new(x_min + offset_mm, y_max - offset_mm),
                    bezier: false,
                },
            ],
            is_closed: true,
        };

        ops.push(Op::DrawLine { line: border });
    }

    fn draw_page_number(
        &self,
        page_index: usize,
        font: &PdfFont,
        l_cfg: &LayoutConfig,
        ops: &mut Vec<Op>,
    ) {
        let font_size = Pt(11.0);
        let mm_to_pt = 72.0 / 25.4;
        let number_text = format!("{}", page_index);

        let text_width = measure_width(&number_text, font, font_size.0);

        let content_width_mm = l_cfg.page_width.0 - l_cfg.margin_left.0 - l_cfg.margin_right.0;
        let content_width_pt = content_width_mm * mm_to_pt;
        let x_pt = (l_cfg.margin_left.0 * mm_to_pt) + (content_width_pt - text_width) / 2.0;

        let y_center_mm = l_cfg.margin_bottom.0 / 2.0;

        let font_correction_pt = font_size.0 * 0.3;
        let y_pt = (y_center_mm * mm_to_pt) - font_correction_pt;

        ops.push(Op::StartTextSection);
        ops.push(Op::SetFont {
            font: font.handle.clone(),
            size: font_size,
        });
        ops.push(Op::SetTextMatrix {
            matrix: TextMatrix::Translate(Pt(x_pt), Pt(y_pt)),
        });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(number_text)],
        });
        ops.push(Op::EndTextSection);
    }

    fn fix_punctuation(&self, text: &str, target: &str) -> String {
        let t = text.trim();
        if t.is_empty() {
            return String::new();
        }

        let last_c = t.chars().last().unwrap();
        if [';', '.', '!', '?', ':'].contains(&last_c) {
            if ['.', ';'].contains(&last_c) {
                let mut s = t[..t.len() - last_c.len_utf8()].to_string();
                s.push_str(target);
                s
            } else {
                t.to_string()
            }
        } else {
            format!("{}{}", t, target)
        }
    }
}

impl Backend for PdfBackend {
    fn render(&mut self, ast_root: &Document) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let l_cfg = LayoutConfig::from_node(ast_root);
        let mut doc = PdfDocument::new("Образец");

        let fonts = self.load_fonts(&mut doc)?;

        let mut state = RenderState::new(&l_cfg);

        for child in &ast_root.children {
            self.collect_ops(child, &l_cfg, &fonts, &mut state);
        }

        if state.show_page_numbers {
            self.draw_page_number(
                state.page_count,
                &fonts.regular,
                &l_cfg,
                &mut state.current_ops,
            );
        }

        state.current_ops.push(Op::EndTextSection);

        let final_ops = std::mem::take(&mut state.current_ops);
        state
            .pages
            .push(PdfPage::new(l_cfg.page_width, l_cfg.page_height, final_ops));

        let mut save_warnings = Vec::new();
        let pdf_bytes = doc
            .with_pages(state.pages)
            .save(&PdfSaveOptions::default(), &mut save_warnings);

        Ok(pdf_bytes)
    }
}
