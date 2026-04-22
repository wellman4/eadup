//  Copyright (C) 2026 Ivan Goglenkov (wellman4)
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License.

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AbstractKind { Extended, Short }

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ContentsKind { Collection, Integrated }

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ConclusionKind { Final, Summary }

#[derive(Debug, PartialEq, Clone, Copy)]
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NoteKind {
    General,
    Example,
    Remark, 
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType<'a> {
    Structural(StructuralKind),
    Section { level: u8, heading: &'a str },
    Figure,
    Row,
    Cell(Option<&'a str>),
    Table,
    Listing(String),
    Attribute { key: &'a str, value: &'a str },
    Note(NoteKind),
    Paragraph(&'a str),
    ListItem { level: u8, text: &'a str },
    Error(String),
    EmptyLine,
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub kind: TokenType<'a>,
    pub line: usize,
    pub raw: &'a str,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenType<'a>, line: usize, raw: &'a str) -> Self {
        Self { kind, line, raw }
    }
}