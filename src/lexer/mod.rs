//  Copyright (C) 2026 wellman4
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License.

pub mod token;
pub use token::{
    AbstractKind, ConclusionKind, ContentsKind, NoteKind, StructuralKind, Token, TokenType,
};

#[derive(Clone)]
pub struct Lexer<'a> {
    lines: std::str::Lines<'a>,
    line_num: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lines: input.lines(),
            line_num: 0,
        }
    }

    fn identify_kind(&mut self, raw: &'a str) -> TokenType<'a> {
        let stripped = raw.trim();

        // Пустая строка
        if stripped.is_empty() {
            return TokenType::EmptyLine;
        }

        // Листинг
        if stripped.starts_with("```") {
            return self.parse_listing();
        }

        // Раздел
        if stripped.starts_with('/') {
            let level = stripped.chars().take_while(|&c| c == '/').count();
            let heading = stripped[level..].trim();
            return TokenType::Section {
                level: level as u8,
                heading,
            };
        }

        // Атрибут
        if stripped.starts_with('\\') {
            return match stripped[1..].split_once('=') {
                Some((key, val)) => TokenType::Attribute {
                    key: key.trim(),
                    value: val.trim(),
                },
                None => TokenType::Error {
                    message: "неверный формат атрибута (ожидалось '\\ключ = значение')".to_string(),
                },
            };
        }

        // Перечисление
        if stripped.starts_with('-') {
            let level = stripped.chars().take_while(|&c| c == '-').count();
            let text = stripped[level..].trim();
            return TokenType::ListItem {
                level: level as u8,
                text,
            };
        }

        if stripped.starts_with("ГРАФА") {
            let content = stripped["ГРАФА".len()..].trim();

            return if content.is_empty() {
                TokenType::Cell { text: None }
            } else {
                TokenType::Cell {
                    text: Some(content),
                }
            };
        }

        // Ключевое слово
        if let Some(kind) = self.match_keyword(stripped) {
            return kind;
        }

        // Абзац
        TokenType::Paragraph { text: stripped }
    }

    fn parse_listing(&mut self) -> TokenType<'a> {
        let start_line = self.line_num;
        let mut code = String::new();
        let mut found_end = false;
        while let Some(line) = self.lines.next() {
            self.line_num += 1;
            if line.trim().starts_with("```") {
                found_end = true;
                break;
            }
            code.push_str(line);
            code.push('\n');
        }
        if found_end {
            TokenType::Listing { code }
        } else {
            TokenType::Error {
                message: format!("незакрытый листинг (начало на строке {})", start_line),
            }
        }
    }

    fn match_keyword(&self, s: &'a str) -> Option<TokenType<'a>> {
        use AbstractKind::*;
        use ConclusionKind::*;
        use ContentsKind::*;
        use NoteKind::*;
        use StructuralKind::*;

        match s {
            "ТИТУЛЬНЫЙ ЛИСТ" => Some(TokenType::Structural { kind: TitlePage }),
            "ЛИСТ ДЛЯ ЗАМЕЧАНИЙ" => Some(TokenType::Structural { kind: NotesPage }),

            "РЕФЕРАТ" => Some(TokenType::Structural {
                kind: Abstract(Extended),
            }),
            "АННОТАЦИЯ" => Some(TokenType::Structural {
                kind: Abstract(Short),
            }),

            "СОДЕРЖАНИЕ" => Some(TokenType::Structural {
                kind: Contents(Collection),
            }),
            "ОГЛАВЛЕНИЕ" => Some(TokenType::Structural {
                kind: Contents(Integrated),
            }),

            "НОРМАТИВНЫЕ ССЫЛКИ" => Some(TokenType::Structural {
                kind: NormativeReferences,
            }),
            "ОПРЕДЕЛЕНИЯ ОБОЗНАЧЕНИЯ СОКРАЩЕНИЯ" => {
                Some(TokenType::Structural { kind: Definitions })
            }
            "ВВЕДЕНИЕ" => Some(TokenType::Structural { kind: Introduction }),
            "ОСНОВНАЯ ЧАСТЬ" => Some(TokenType::Structural { kind: MainPart }),

            "ЗАКЛЮЧЕНИЕ" => Some(TokenType::Structural {
                kind: Conclusion(Final),
            }),
            "ВЫВОДЫ" => Some(TokenType::Structural {
                kind: Conclusion(Summary),
            }),

            "СПИСОК ИСПОЛЬЗОВАННЫХ ИСТОЧНИКОВ" => {
                Some(TokenType::Structural { kind: Sources })
            }
            "ПРИЛОЖЕНИЕ" => Some(TokenType::Structural { kind: Appendix }),
            "СВЕДЕНИЯ О САМОСТОЯТЕЛЬНОСТИ ВЫПОЛНЕНИЯ РАБОТЫ" => {
                Some(TokenType::Structural {
                    kind: IndependenceStatement,
                })
            }

            "РИСУНОК" => Some(TokenType::Figure),
            "ТАБЛИЦА" => Some(TokenType::Table),
            "СТРОКА" => Some(TokenType::Row),

            "ПРИМЕЧАНИЯ" => Some(TokenType::Note { kind: General }),
            "ПРИМЕРЫ" => Some(TokenType::Note { kind: Example }),
            "ПОЯСНЕНИЯ" => Some(TokenType::Note { kind: Remark }),

            _ => None,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let raw = self.lines.next()?;
        self.line_num += 1;

        let kind = self.identify_kind(raw);
        Some(Token::new(kind, self.line_num, raw))
    }
}
