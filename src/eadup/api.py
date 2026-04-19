#  Copyright (C) 2026 Ivan Goglenkov (wellman4) 
#  This program is free software: you can redistribute it and/or modify
#  it under the terms of the GNU Affero General Public License as published by
#  the Free Software Foundation, either version 3 of the License.

import io
from lexer import Lexer
from parser import Parser
from backend.pdf import PDFBackend

def compile_string_to_pdf(source_code: str) -> bytes:
    lexer = Lexer(io.StringIO(source_code))
    parser = Parser()
    ast_root = parser.parse(lexer)
    backend = PDFBackend()
    return backend.generate(ast_root)
