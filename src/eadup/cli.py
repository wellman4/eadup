#  Copyright (C) 2026 Ivan Goglenkov (wellman4)
#  This program is free software: you can redistribute it and/or modify
#  it under the terms of the GNU Affero General Public License as published by
#  the Free Software Foundation, either version 3 of the License.

#!/usr/bin/env python
import sys
from pathlib import Path
import importlib

from .lexer import Lexer
from .parser import Parser

BACKENDS = {
    'pdf':  ('pdf',  'PDFBackend',  '.pdf'),
    'html': ('html', 'HTMLBackend', '.html'),
    'json': ('json', 'JSONBackend', '.json'),
}

def load_backend(format_name: str):
    if format_name not in BACKENDS:
        raise ValueError(f"Неизвестный формат: {format_name}")
    module_name, class_name, _ = BACKENDS[format_name]
    module = importlib.import_module(f'eadup.backend.{module_name}')
    backend_class = getattr(module, class_name)
    return backend_class()

def main():
    import argparse
    arg_parser = argparse.ArgumentParser(description="Компилятор EADUP")
    arg_parser.add_argument("input", help="Входной файл (.eqd)")
    arg_parser.add_argument("-f", "--format", default="pdf", choices=list(BACKENDS.keys()),
                            help="Выходной формат (pdf, html, json)")
    args = arg_parser.parse_args()

    try:
        with open(args.input, 'r', encoding='utf-8') as f:
            lexer = Lexer(f)
            parser = Parser()

            try:
                ast_root = parser.parse(lexer)
            except Exception as e:
                print(f"Ошибка синтаксического анализа: {e}", file=sys.stderr)
                sys.exit(1)

    except FileNotFoundError:
        print(f"Ошибка: Файл {args.input} не найден.", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Непредвиденная ошибка: {e}", file=sys.stderr)
        sys.exit(1)

    input_path = Path(args.input).resolve()
    output_ext = BACKENDS[args.format][2]
    output_path = input_path.with_suffix(output_ext)
    root_dir = input_path.parent

    try:
        backend = load_backend(args.format)
        
        if hasattr(backend, 'root_dir'):
            backend.root_dir = root_dir
            
        output_data = backend.generate(ast_root)
        
    except Exception as e:
        print(f"Ошибка генерации: {e}", file=sys.stderr)
        import traceback; traceback.print_exc()
        sys.exit(1)

    try:
        if isinstance(output_data, bytes):
            output_path.write_bytes(output_data)
        else:
            output_path.write_text(output_data, encoding='utf-8')
    except Exception as e:
        print(f"Ошибка записи: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == '__main__':
    main()
