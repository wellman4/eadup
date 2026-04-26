//  Copyright (C) 2026 wellman4
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License.

use wasm_bindgen::prelude::*;

pub mod lexer;
pub mod parser;
pub mod backend;

use backend::Backend;

#[wasm_bindgen]
pub fn compile(content: &str) -> Result<Vec<u8>, JsError> {
    let lexer = lexer::Lexer::new(content);

    let mut parser = parser::Parser::new(lexer);
    let document_ast = parser.parse();
    
    let parser::ast::Node::Document(ref doc) = document_ast else {
        return Err(JsError::new("Корень AST должен быть документом"));
    };

    let mut pdf_backend = backend::pdf::PdfBackend::new();
    pdf_backend.render(doc)
        .map_err(|e| JsError::new(&e.to_string()))
}