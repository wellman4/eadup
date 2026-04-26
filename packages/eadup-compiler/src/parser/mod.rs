//  Copyright (C) 2026 wellman4
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License.

pub mod ast;

use std::collections::HashMap;
use crate::lexer::token::{Token, TokenType};
use crate::parser::ast::*;

pub struct Parser<'a, I> 
where 
    I: Iterator<Item = Token<'a>> 
{
    tokens: std::iter::Peekable<I>,
    current_attributes: HashMap<String, String>,
}

impl<'a, I> Parser<'a, I> 
where 
    I: Iterator<Item = Token<'a>> 
{
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
            current_attributes: HashMap::new(),
        }
    }

    pub fn parse(&mut self) -> Node {
        let mut root = Document {
            children: Vec::new(),
            attributes: HashMap::new(),
        };

        let mut stack: Vec<Node> = Vec::new();

        while let Some(token) = self.tokens.next() {
            match &token.kind {                
                TokenType::EmptyLine => self.handle_empty_line(&mut stack, &mut root),
                
                TokenType::Attribute { key, value } => {
                    if let Some(top) = stack.last_mut() {
                        self.apply_to_node(top, key, value);
                    } else {
                        root.attributes.insert(key.to_string(), value.to_string());
                    }
                }

                _ => {
                    if let Some(mut new_node) = self.create_node_instance(&token) {
                        self.apply_attributes(&mut new_node);
                        self.process_token(new_node, &mut stack, &mut root, &token.kind);
                    }
                }
            }
        }

        self.close_stack_to_bottom(&mut stack, &mut root);
        Node::Document(root)
    }

    fn process_token(&mut self, node: Node, stack: &mut Vec<Node>, root: &mut Document, kind: &TokenType) {
        if let Node::Structural(_) = node {
            while let Some(top) = stack.last() {
                if matches!(top, Node::Structural(_) | Node::Section(_) | Node::Note(_) | Node::Table(_) | Node::List(_)) {
                    let closed = stack.pop().unwrap();
                    self.add_to_parent(closed, stack, root);
                } else { break; }
            }
        }

        if let Node::ListItem(item) = node {
            self.process_list_item(item.level, item.text, stack, root);
            return;
        }

        if let Node::Section(ref new_sec) = node {
            while let Some(top) = stack.last() {
                let should_pop = match top {
                    Node::Section(s) => s.level >= new_sec.level,
                    Node::Table(_) | Node::Figure(_) | Node::Note(_) | Node::List(_) => true,
                    _ => false,
                };
                if should_pop {
                    let closed = stack.pop().unwrap();
                    self.add_to_parent(closed, stack, root);
                } else { break; }
            }
        }

        while !stack.is_empty() {
            let top = stack.last().unwrap();
            
            let is_new_structural = matches!(kind, TokenType::Structural { .. });
            let top_is_structural = matches!(top, Node::Structural(_));

            if !self.can_contain(top, kind) || (is_new_structural && top_is_structural) {
                let closed = stack.pop().unwrap();
                self.add_to_parent(closed, stack, root);
            } else {
                break;
            }
        }

        if self.is_container(&node) {
            stack.push(node);
        } else {
            self.add_to_parent(node, stack, root);
        }
    }

    fn process_list_item(&mut self, level: u8, text: String, stack: &mut Vec<Node>, root: &mut Document) {
        while let Some(top) = stack.last() {
            let should_pop = match top {
                Node::ListItem(i) => i.level >= level,
                Node::List(_) => {
                    if stack.len() >= 2 {
                        if let Node::ListItem(parent_item) = &stack[stack.len() - 2] {
                            parent_item.level >= level
                        } else { false }
                    } else { false }
                },
                _ => !self.is_container(top),
            };

            if should_pop {
                let closed = stack.pop().unwrap();
                self.add_to_parent(closed, stack, root);
            } else {
                break;
            }
        }

        let needs_new_list = match stack.last() {
            Some(Node::List(_)) => false,
            Some(Node::ListItem(i)) if i.level < level => true,
            _ => true,
        };

        if needs_new_list {
            let new_list = Node::List(ListContainer { items: Vec::new() });
            stack.push(new_list);
        }

        let new_item = Node::ListItem(ListItem {
            level,
            text,
            children: Vec::new(),
        });
        stack.push(new_item);
    }

    fn add_to_parent(&self, node: Node, stack: &mut Vec<Node>, root: &mut Document) {
        if let Some(parent) = stack.last_mut() {
            match (parent, &node) {
                (Node::List(l), Node::ListItem(_)) => l.items.push(node),
                
                (_, Node::ListItem(_)) => {
                    root.children.push(node);
                }

                (Node::Document(d), _) => d.children.push(node),
                (Node::Structural(s), _) => s.children.push(node),
                (Node::Section(s), _) => s.children.push(node),
                (Node::ListItem(i), _) => i.children.push(node),
                (Node::Figure(f), _) => f.children.push(node),
                (Node::Table(t), _) => t.children.push(node),
                (Node::TableRow(r), _) => r.cells.push(node),
                (Node::TableCell(c), _) => c.children.push(node),
                (Node::Note(n), _) => n.children.push(node),
                
                _ => root.children.push(node),
            }
        } else {
            root.children.push(node);
        }
    }

    fn handle_empty_line(&mut self, stack: &mut Vec<Node>, root: &mut Document) {
        while let Some(top) = stack.last() {
            let should_pop = matches!(top, 
                Node::TableCell(_) | Node::TableRow(_) | Node::Table(_) | 
                Node::ListItem(_) | Node::List(_) | Node::Note(_) | Node::Figure(_)
            );
            if should_pop {
                let node = stack.pop().unwrap();
                self.add_to_parent(node, stack, root);
            } else { break; }
        }
    }

    fn apply_attributes(&mut self, node: &mut Node) {
        if self.current_attributes.is_empty() { return; }
        match node {
            Node::Document(d) => d.attributes.extend(self.current_attributes.drain()),
            Node::Structural(s) => s.attributes.extend(self.current_attributes.drain()),
            Node::Table(t) => t.attributes.extend(self.current_attributes.drain()),
            Node::Listing(l) => l.attributes.extend(self.current_attributes.drain()),
            Node::Figure(f) => f.attributes.extend(self.current_attributes.drain()),
            _ => {} 
        }
    }

    fn apply_to_node(&self, node: &mut Node, key: &str, value: &str) {
        match node {
            Node::Structural(s) => { s.attributes.insert(key.into(), value.into()); }
            Node::Table(t) => { t.attributes.insert(key.into(), value.into()); }
            Node::Listing(l) => { l.attributes.insert(key.into(), value.into()); }
            Node::Figure(f) => {
                f.attributes.insert(key.to_string(), value.to_string());
            }            
            Node::Document(d) => { d.attributes.insert(key.into(), value.into()); }
            _ => {}
        }
    }

    fn close_stack_to_bottom(&self, stack: &mut Vec<Node>, root: &mut Document) {
        while let Some(node) = stack.pop() {
            self.add_to_parent(node, stack, root);
        }
    }

    fn is_container(&self, node: &Node) -> bool {
        matches!(node, Node::Structural(_) | Node::Section(_) | Node::Table(_) | 
                 Node::TableRow(_) | Node::TableCell(_) | Node::Note(_) | 
                 Node::List(_) | Node::ListItem(_) | Node::Figure(_) |
                 Node::Listing(_))
    }

    fn create_node_instance(&self, token: &Token<'a>) -> Option<Node> {
        match &token.kind {
            TokenType::Paragraph { text } => Some(Node::Paragraph(Paragraph { text: text.to_string() })),
            TokenType::Section { level, heading } => Some(Node::Section(Section {
                level: *level,
                title: heading.to_string(),
                children: Vec::new(),
            })),
            TokenType::ListItem { level, text } => Some(Node::ListItem(ListItem {
                level: *level,
                text: text.to_string(),
                children: Vec::new(),
            })),
            TokenType::Table => Some(Node::Table(Table { children: Vec::new(), attributes: HashMap::new() })),
            TokenType::Row => Some(Node::TableRow(TableRow { cells: Vec::new() })),
            TokenType::Cell { text } => {
                let mut cell = TableCell { children: Vec::new() };
                if let Some(txt) = text {
                    cell.children.push(Node::Paragraph(Paragraph { text: txt.to_string() }));
                }
                Some(Node::TableCell(cell))
            },
            TokenType::Listing { code } => Some(Node::Listing(Listing { text: code.clone(), attributes: HashMap::new() })),
            TokenType::Note { kind } => Some(Node::Note(NoteBlock { kind: *kind, children: Vec::new() })),
            TokenType::Figure => Some(Node::Figure(Attributed {
                attributes: HashMap::new(),
                children: Vec::new(),
            })),
            TokenType::Structural { kind } => Some(Node::Structural(StructuralElement {
                kind: *kind,
                children: Vec::new(),
                attributes: HashMap::new(),
            })),
            _ => None,
        }
    }

    fn can_contain(&self, parent: &Node, child_kind: &TokenType) -> bool {
        match parent {
            Node::Document(_) => matches!(child_kind, 
                TokenType::Structural { .. } | TokenType::Attribute { .. } | TokenType::EmptyLine
            ),

            Node::Structural(_) => matches!(child_kind, 
                TokenType::Paragraph { .. } | TokenType::Section { .. } | 
                TokenType::Figure | TokenType::Table | 
                TokenType::ListItem { .. } | TokenType::Note { .. } | 
                TokenType::Attribute { .. } | TokenType::EmptyLine | 
                TokenType::Listing { .. }
            ),

            Node::Section(_) => matches!(child_kind, 
                TokenType::Paragraph { .. } | TokenType::ListItem { .. } | 
                TokenType::Figure | TokenType::Table | 
                TokenType::Listing { .. } | TokenType::Note { .. } | 
                TokenType::Section { .. }
            ),

            Node::Table(_) => matches!(child_kind, TokenType::Row | TokenType::Attribute { .. } | TokenType::Note { .. } | TokenType::EmptyLine),
            
            Node::TableRow(_) => matches!(child_kind, TokenType::Cell { .. }),
            
            Node::TableCell(_) => matches!(child_kind, 
                TokenType::Paragraph { .. } | TokenType::ListItem { .. } | TokenType::EmptyLine
            ),

            Node::Note(_) => matches!(child_kind, TokenType::ListItem { .. }),

            Node::Figure(_) => matches!(child_kind, TokenType::Attribute { .. } | TokenType::Note { .. } | TokenType::EmptyLine),

            Node::List(_) => matches!(child_kind, TokenType::ListItem { .. }),

            Node::ListItem(_) => matches!(child_kind, TokenType::Paragraph { .. } | TokenType::EmptyLine),

            Node::Listing(_) => matches!(child_kind, TokenType::Attribute { .. }),

            _ => false,
        }
    }
}

