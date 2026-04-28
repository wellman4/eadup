//  Copyright (C) 2026 wellman4
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License.

use serde::Serialize;

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AbstractKind {
    Extended,
    Short,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ContentsKind {
    Collection,
    Integrated,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConclusionKind {
    Final,
    Summary,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StructuralKind {
    TitlePage,
    NotesPage,
    Abstract(AbstractKind),
    Contents(ContentsKind),
    NormativeReferences,
    Definitions,
    Introduction,
    MainPart,
    Conclusion(ConclusionKind),
    Sources,
    Appendix,
    IndependenceStatement,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub enum NoteKind {
    General,
    Example,
    Remark,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TokenType<'a> {
    Structural {
        kind: StructuralKind,
    },
    Section {
        level: u8,
        heading: &'a str,
    },
    Figure,
    Row,
    Cell {
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<&'a str>,
    },
    Table,
    Listing {
        code: String,
    },
    Attribute {
        key: &'a str,
        value: &'a str,
    },
    Note {
        kind: NoteKind,
    },
    Paragraph {
        text: &'a str,
    },
    ListItem {
        level: u8,
        text: &'a str,
    },
    Error {
        message: String,
    },
    EmptyLine,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Token<'a> {
    pub kind: TokenType<'a>,
    pub line: usize,
    #[serde(skip_serializing)]
    pub raw: &'a str,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenType<'a>, line: usize, raw: &'a str) -> Self {
        Self { kind, line, raw }
    }
}
