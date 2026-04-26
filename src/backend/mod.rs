//  Copyright (C) 2026 Ivan Goglenkov (wellman4)
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License.

pub mod pdf;

pub trait Backend {
    fn render(&mut self, doc: &crate::parser::ast::Document) -> Result<String, Box<dyn std::error::Error>>;
}