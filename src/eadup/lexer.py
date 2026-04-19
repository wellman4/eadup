#  Copyright (C) 2026 Ivan Goglenkov (wellman4)
#  This program is free software: you can redistribute it and/or modify
#  it under the terms of the GNU Affero General Public License as published by
#  the Free Software Foundation, either version 3 of the License.

from typing import Iterable, Iterator
import re
from .tokens import Token, TokenType
from .elements import STRUCTURAL_ELEMENTS, SYNONYMS

TRIGGER_MAP = {
    'ТАБЛИЦА': TokenType.TABLE,
    'РИСУНОК': TokenType.FIGURE,
    'ПРИМЕЧАНИЯ': TokenType.NOTE,
    'ПРИМЕРЫ': TokenType.NOTE,
    'СТРОКА': TokenType.ROW,
    'ГРАФА': TokenType.CELL,
    'ПОЯСНЕНИЯ': TokenType.NOTE,
    'ЛИСТИНГ': TokenType.LISTING
}

RE_HEADING = re.compile(r'^(?P<lvl>/+)\s*(?P<title>.*)$')
RE_ATTR = re.compile(r'^([^\s=]+)\s*=\s*(.*)$')
RE_LIST = re.compile(r'^(?P<lvl>--?)\s+(?P<text>.*)$')

class Lexer:
    def __init__(self, lines: Iterable[str]):
        self.lines = lines
        self._line_num = 0

    def __iter__(self) -> Iterator[Token]:
        it = iter(self.lines)
        
        while True:
            try:
                raw_line = next(it)
                self._line_num += 1
                
                clean_raw = raw_line.rstrip('\r\n')
                stripped = clean_raw.strip()

                if stripped.startswith('```'):
                    content = []
                    start_line = self._line_num
                    
                    while True:
                        try:
                            code_line = next(it)
                            self._line_num += 1
                            if code_line.strip().startswith('```'):
                                break
                            content.append(code_line.rstrip('\r\n'))
                        except StopIteration:
                            break
                    
                    full_text = "\n".join(content)
                    yield Token(
                        type=TokenType.LISTING, 
                        line=start_line, 
                        value=full_text, 
                        payload={'text': full_text}
                    )
                    continue

                token = self._identify_line(clean_raw, self._line_num)
                if token:
                    yield token

            except StopIteration:
                break

        yield Token(type=TokenType.EOF, line=self._line_num, value="")

    def _identify_line(self, raw_line: str, line_num: int) -> Token:
        stripped = raw_line.strip()
        
        if not stripped:
            return Token(type=TokenType.EMPTY_LINE, line=line_num, value=raw_line)
    
        if stripped.startswith('ГРАФА'):
            parts = stripped.split(maxsplit=1)
            content = parts[1] if len(parts) > 1 else ""
            
            return Token(
                type=TokenType.CELL, 
                line=line_num, 
                value=raw_line, 
                payload={'text': content}
            )

        if stripped in TRIGGER_MAP:
            token_type = TRIGGER_MAP[stripped]

            if stripped in {'ПРИМЕЧАНИЯ', 'ПРИМЕРЫ', 'ПОЯСНЕНИЯ'}:
                return Token(
                    type=token_type, 
                    line=line_num, 
                    value=raw_line, 
                    payload={'kind': stripped}
                )
        
            return Token(type=TRIGGER_MAP[stripped], line=line_num, value=raw_line, payload={'name': stripped})
        
        if stripped in STRUCTURAL_ELEMENTS:
            canonical_name = SYNONYMS.get(stripped, stripped)
            return Token(
                type=TokenType.STRUCTURAL,
                line=line_num,
                value=raw_line,
                payload={
                    'name': canonical_name,
                    'display_name': stripped
                }
            )

        first_char = stripped[0]
        
        if first_char == '/':
            m = RE_HEADING.match(stripped)
            if m:
                return Token(type=TokenType.HEADING, line=line_num, 
                            value=raw_line,
                            payload={'level': len(m.group('lvl')), 'title': m.group('title')})

        if first_char == '\\':
            m = RE_ATTR.match(stripped[1:].lstrip())
            if m:
                return Token(type=TokenType.ATTRIBUTE, line=line_num, 
                            value=raw_line, 
                            payload={'key': m.group(1), 'text': m.group(2).strip()})
            return Token(type=TokenType.ERROR, line=line_num, value=raw_line, payload={'msg': 'Invalid attr'})

        if first_char == '-':
            m = RE_LIST.match(stripped)
            if m:
                return Token(type=TokenType.LIST_ITEM, line=line_num, 
                            value=raw_line, 
                            payload={'level': len(m.group('lvl')), 'text': m.group('text')})

        return Token(type=TokenType.PARAGRAPH, line=line_num, value=raw_line, payload={'text': stripped})
