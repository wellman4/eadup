//  Copyright (C) 2026 wellman4
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License.

use crate::lexer::token::{NoteKind, StructuralKind};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Node {
    Document(Document),
    Structural(StructuralElement),
    Section(Section),
    Paragraph(Paragraph),
    List(ListContainer),
    ListItem(ListItem),
    Figure(Attributed),
    Table(Table),
    TableRow(TableRow),
    TableCell(TableCell),
    Listing(Listing),
    Note(NoteBlock),
}

impl Node {
    pub fn as_document(&self) -> Option<&Document> {
        if let Node::Document(d) = self {
            Some(d)
        } else {
            None
        }
    }

    pub fn debug_print(&self, indent: usize) -> String {
        let mut result = String::new();
        let space = "  ".repeat(indent);

        let truncate = |text: &str| -> String {
            if text.chars().count() > 40 {
                text.chars().take(37).collect::<String>() + "..."
            } else {
                text.to_string()
            }
        };

        match self {
            Node::Document(doc) => {
                writeln!(result, "{}Document [attrs: {:?}]", space, doc.attributes).unwrap();
                for child in &doc.children {
                    result.push_str(&child.debug_print(indent + 1));
                }
            }
            Node::Structural(el) => {
                writeln!(
                    result,
                    "{}Structural(kind={:?}) [attrs: {:?}]",
                    space, el.kind, el.attributes
                )
                .unwrap();
                for child in &el.children {
                    result.push_str(&child.debug_print(indent + 1));
                }
            }
            Node::Section(h) => {
                writeln!(
                    result,
                    "{}Section(level={}, title='{}')",
                    space, h.level, h.title
                )
                .unwrap();
                for child in &h.children {
                    result.push_str(&child.debug_print(indent + 1));
                }
            }
            Node::Paragraph(p) => {
                writeln!(result, "{}Paragraph(text='{}')", space, truncate(&p.text)).unwrap();
            }
            Node::List(l) => {
                writeln!(result, "{}List", space).unwrap();
                for item in &l.items {
                    result.push_str(&item.debug_print(indent + 1));
                }
            }
            Node::ListItem(item) => {
                writeln!(
                    result,
                    "{}ListItem(level={}, text='{}')",
                    space,
                    item.level,
                    truncate(&item.text)
                )
                .unwrap();
                for child in &item.children {
                    result.push_str(&child.debug_print(indent + 1));
                }
            }
            Node::Table(t) => {
                let row_count = t
                    .children
                    .iter()
                    .filter(|n| matches!(n, Node::TableRow(_)))
                    .count();
                writeln!(
                    result,
                    "{}Table [rows: {}, attrs: {:?}]",
                    space, row_count, t.attributes
                )
                .unwrap();
                for child in &t.children {
                    result.push_str(&child.debug_print(indent + 1));
                }
            }
            Node::TableRow(row) => {
                writeln!(result, "{}TableRow", space).unwrap();
                for cell in &row.cells {
                    result.push_str(&cell.debug_print(indent + 1));
                }
            }
            Node::TableCell(cell) => {
                writeln!(result, "{}TableCell", space).unwrap();
                for child in &cell.children {
                    result.push_str(&child.debug_print(indent + 1));
                }
            }
            Node::Note(n) => {
                writeln!(result, "{}Note(kind={:?})", space, n.kind).unwrap();
                for child in &n.children {
                    result.push_str(&child.debug_print(indent + 1));
                }
            }
            Node::Listing(l) => {
                writeln!(
                    result,
                    "{}Listing(text='{}') [attrs: {:?}]",
                    space,
                    truncate(&l.text.replace('\n', " ")),
                    l.attributes
                )
                .unwrap();
            }
            Node::Figure(f) => {
                writeln!(result, "{}Figure [attrs: {:?}]", space, f.attributes).unwrap();
                for child in &f.children {
                    result.push_str(&child.debug_print(indent + 1));
                }
            }
        }
        result
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub children: Vec<Node>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralElement {
    pub kind: StructuralKind,
    pub children: Vec<Node>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub level: u8,
    pub title: String,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Paragraph {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListContainer {
    pub items: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    pub level: u8,
    pub text: String,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    pub children: Vec<Node>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRow {
    pub cells: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableCell {
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Listing {
    pub text: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteBlock {
    pub kind: NoteKind,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributed {
    pub attributes: HashMap<String, String>,
    pub children: Vec<Node>,
}
