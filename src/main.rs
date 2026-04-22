//  Copyright (C) 2026 Ivan Goglenkov (wellman4)
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License.

use std::fs;
use std::path::PathBuf;
use std::process;
use clap::{Parser, ValueEnum};

mod lexer;

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
    Docx,
}

fn main() {
    let args = Args::parse();

    // Чтение файла
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
        output.push_str(&format!("{:<5} | {:<30} | TOKEN KIND\n", "LINE", "RAW"));
        output.push_str(&format!("{:-<80}\n", ""));

        for token in lexer.clone() {
            if let lexer::token::TokenType::Error(ref msg) = token.kind {
                eprintln!("предупреждение на строке {}: {}", token.line, msg);
            }

            let raw_trimmed = if token.raw.chars().count() > 30 {
                let mut s: String = token.raw.chars().take(27).collect();
                s.push_str("...");
                s
            } else {
                token.raw.to_string()
            };

            output.push_str(&format!(
                "{:<5} | {:<30} | {:?}\n", 
                token.line, 
                raw_trimmed.replace('\n', "\\n"),
                token.kind
            ));
        }

        if let Err(e) = fs::write(&output_path, output) {
            eprintln!("Ошибка записи токенов: {}", e);
            process::exit(1);
        }
        println!("Токены записаны в {:?}", output_path);
    }
}
