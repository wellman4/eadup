//  Copyright (C) 2026 wellman4
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License.

pub mod backend;
pub mod lexer;
pub mod parser;

use crate::backend::Backend;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum OutputFormat {
    Pdf,
}

pub fn get_tokens(content: &str) -> Vec<lexer::token::Token<'_>> {
    lexer::Lexer::new(content).collect()
}

pub fn get_ast(content: &str) -> parser::ast::Node {
    let lexer = lexer::Lexer::new(content);
    let mut parser = parser::Parser::new(lexer);
    parser.parse()
}

pub fn render(
    ast: &parser::ast::Node,
    format: OutputFormat,
) -> Result<(Vec<u8>, &'static str), Box<dyn std::error::Error>> {
    let doc = ast.as_document().ok_or("Root is not a document")?;

    match format {
        OutputFormat::Pdf => {
            let mut backend = backend::pdf::PdfBackend::new();
            let bytes = backend.render(doc)?;
            Ok((bytes, "pdf"))
        }
    }
}

pub fn compile(
    content: &str,
    output_format: OutputFormat,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let ast = get_ast(content);
    let (bytes, _) = render(&ast, output_format)?;
    Ok(bytes)
}

#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = compile)]
pub fn compile_wasm(content: &str) -> Result<Vec<u8>, JsError> {
    compile(content, OutputFormat::Pdf).map_err(|e| JsError::new(&e.to_string()))
}
