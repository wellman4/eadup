//  Copyright (C) 2026 wellman4
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License.

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use std::process;

#[cfg(feature = "cli")]
use clap::Parser;

use eadup::OutputFormat;
#[cfg(feature = "cli")]
use eadup::lexer::token::TokenType;

#[cfg(feature = "cli")]
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the input file (.ead)
    #[arg(value_name = "FILE")]
    input: PathBuf,

    /// Output file format
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Pdf)]
    format: OutputFormat,

    /// Save the lexical token stream to a file
    #[arg(long)]
    emit_tokens: bool,

    /// Save the Abstract Syntax Tree (AST) structure to a file
    #[arg(long)]
    emit_ast: bool,
}

#[cfg(feature = "cli")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Read input file content
    let content = match fs::read_to_string(&args.input) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: could not read file {:?}: {}", args.input, e);
            process::exit(1);
        }
    };

    if args.emit_tokens {
        let tokens = eadup::get_tokens(&content);
        let output_path = args.input.with_extension("tokens");
        let mut output = String::new();

        writeln!(output, "{:<5} | {:<30} | TOKEN KIND", "LINE", "RAW")?;
        writeln!(output, "{:-<80}", "")?;

        for token in tokens {
            if let TokenType::Error { ref message } = token.kind {
                eprintln!("Warning at line {}: {}", token.line, message);
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
            eprintln!("Error writing tokens file: {}", e);
            e
        })?;
    }

    let ast = eadup::get_ast(&content);

    if args.emit_ast {
        let debug_tree = ast.debug_print(0);
        fs::write(args.input.with_extension("ast"), debug_tree)?;
    }

    let (output_bytes, extension) = eadup::render(&ast, args.format)?;
    let output_path = args.input.with_extension(extension);
    fs::write(&output_path, output_bytes)?;
    println!("Document generated at {:?}", output_path);

    Ok(())
}
