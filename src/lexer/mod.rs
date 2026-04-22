//  Copyright (C) 2026 Ivan Goglenkov (wellman4)
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License.

pub mod token;
pub use token::{
    Token, TokenType, NoteKind, StructuralKind, 
    AbstractKind, ContentsKind, ConclusionKind
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
            return TokenType::Section { level: level as u8, heading };
        }

        // Атрибут
        if stripped.starts_with('\\') {
            return match stripped[1..].split_once('=') {
                Some((key, val)) => TokenType::Attribute { key: key.trim(), value: val.trim() },
                None => TokenType::Error("неверный формат атрибута (ожидалось '\\ключ = значение')".to_string()),
            };
        }

        // Перечисление
        if stripped.starts_with('-') {
            let level = stripped.chars().take_while(|&c| c == '-').count();
            let text = stripped[level..].trim();
            return TokenType::ListItem { level: level as u8, text };
        }

        if stripped.starts_with("ГРАФА") {
            let content = stripped["ГРАФА".len()..].trim();
            
            return if content.is_empty() {
                TokenType::Cell(None)
            } else {
                TokenType::Cell(Some(content))
            };
        }

        // Ключевое слово
        if let Some(kind) = self.match_keyword(stripped) {
            return kind;
        }   

        // Абзац
        TokenType::Paragraph(stripped)
    }

    fn parse_listing(&mut self) -> TokenType<'a> {
        let start_line = self.line_num;
        let mut content = String::new();
        let mut found_end = false;
        while let Some(line) = self.lines.next() {
            self.line_num += 1;
            if line.trim().starts_with("```") {
                found_end = true;
                break;
            }
            content.push_str(line);
            content.push('\n');
        }
        if found_end {
            TokenType::Listing(content)
        } else {
            TokenType::Error(format!(
                "незакрытый листинг (начало на строке {})", 
                start_line
            ))
        }
    }

    fn match_keyword(&self, s: &'a str) -> Option<TokenType<'a>> {
        use StructuralKind::*;
        use AbstractKind::*;
        use ContentsKind::*;
        use ConclusionKind::*;
        use NoteKind::*;

        match s {
            "ТИТУЛЬНЫЙ ЛИСТ" => Some(TokenType::Structural(TitlePage)),
            "ЛИСТ ДЛЯ ЗАМЕЧАНИЙ" => Some(TokenType::Structural(NotesPage)),
            
            "РЕФЕРАТ" => Some(TokenType::Structural(Abstract(Extended))),
            "АННОТАЦИЯ" => Some(TokenType::Structural(Abstract(Short))),
            
            "СОДЕРЖАНИЕ" => Some(TokenType::Structural(Contents(Collection))),
            "ОГЛАВЛЕНИЕ" => Some(TokenType::Structural(Contents(Integrated))),
            
            "НОРМАТИВНЫЕ ССЫЛКИ" => Some(TokenType::Structural(NormativeReferences)),
            "ОПРЕДЕЛЕНИЯ ОБОЗНАЧЕНИЯ СОКРАЩЕНИЯ" => Some(TokenType::Structural(Definitions)),
            "ВВЕДЕНИЕ" => Some(TokenType::Structural(Introduction)),
            "ОСНОВНАЯ ЧАСТЬ" => Some(TokenType::Structural(MainPart)),
            
            "ЗАКЛЮЧЕНИЕ" => Some(TokenType::Structural(Conclusion(Final))),
            "ВЫВОДЫ" => Some(TokenType::Structural(Conclusion(Summary))),
            
            "СПИСОК ИСПОЛЬЗОВАННЫХ ИСТОЧНИКОВ" => Some(TokenType::Structural(Sources)),
            "ПРИЛОЖЕНИЕ" => Some(TokenType::Structural(Appendix)),
            "СВЕДЕНИЯ О САМОСТОЯТЕЛЬНОСТИ ВЫПОЛНЕНИЯ РАБОТЫ" => Some(TokenType::Structural(IndependenceStatement)),

            "РИСУНОК" => Some(TokenType::Figure),
            "ТАБЛИЦА" => Some(TokenType::Table),
            "СТРОКА" => Some(TokenType::Row),

            "ПРИМЕЧАНИЯ" => Some(TokenType::Note(General)),
            "ПРИМЕРЫ" => Some(TokenType::Note(Example)),
            "ПОЯСНЕНИЯ" => Some(TokenType::Note(Remark)),

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
 