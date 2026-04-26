//  Copyright (C) 2026 Ivan Goglenkov (wellman4)
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License.

use std::fs;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::process;
use clap::{Parser, ValueEnum};

mod lexer;
mod parser;
mod backend;

use backend::Backend;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Путь к входному файлу (.ead)
    #[arg(value_name = "FILE")]
    input: PathBuf,

    /// Формат выходного файла
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Pdf)]
    format: OutputFormat,

    /// Сохранить результат лексического анализа в .tokens файл
    #[arg(long)]
    emit_tokens: bool,

    /// Сохранить структуру документа в .ast файл
    #[arg(long)]
    emit_ast: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum OutputFormat {
    Pdf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let content = match fs::read_to_string(&args.input) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Ошибка: не удалось прочитать файл {:?}: {}", args.input, e);
            process::exit(1);
        }
    };

    let lexer = lexer::Lexer::new(&content);

    if args.emit_tokens {
        let output_path = args.input.with_extension("tokens");
        let mut output = String::new();

        writeln!(output, "{:<5} | {:<30} | TOKEN KIND", "LINE", "RAW")?;
        writeln!(output, "{:-<80}", "")?;

        for token in lexer.clone() {
            if let lexer::token::TokenType::Error { ref message } = token.kind {
                eprintln!("предупреждение на строке {}: {}", token.line, message);
            }

            let escaped_raw = token.raw.replace('\n', "\\n");
            
            let display_raw = if escaped_raw.chars().count() > 30 {
                format!("{}...", escaped_raw.chars().take(27).collect::<String>())
            } else {
                escaped_raw
            };

            writeln!(
                output,
                "{:<5} | {:<30} | {:?}",
                token.line, display_raw, token.kind
            )?;
        }

        fs::write(&output_path, output).map_err(|e| {
            eprintln!("Ошибка записи токенов: {}", e);
            e
        })?;
    }

    let mut parser = parser::Parser::new(lexer);
    let document_ast = parser.parse();
    let doc = if let parser::ast::Node::Document(ref d) = document_ast {
        d
    } else {
        panic!("AST root must be a Document");
    };

    if args.emit_ast {
        let debug_tree = document_ast.debug_print(0);
        fs::write(args.input.with_extension("ast"), debug_tree)?;
    }
    
    match args.format {
        OutputFormat::Pdf => {
            let mut pdf_backend = backend::pdf::PdfBackend::new();
            pdf_backend.render(doc)?;
        }
    }

    Ok(())
}
