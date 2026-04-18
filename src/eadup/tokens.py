#  Copyright (C) 2026 Ivan Goglenkov
#  This program is free software: you can redistribute it and/or modify
#  it under the terms of the GNU Affero General Public License as published by
#  the Free Software Foundation, either version 3 of the License.

from enum import Enum, auto
from dataclasses import dataclass, field
from typing import Mapping, Any


class TokenType(Enum):
    STRUCTURAL = auto()
    HEADING = auto()      
    TABLE = auto()         
    FIGURE = auto()          
    ROW = auto()          
    CELL = auto()            
    LIST_ITEM = auto()       
    ATTRIBUTE = auto()       
    NOTE = auto()            
    PARAGRAPH = auto()       
    LISTING = auto()
    EMPTY_LINE = auto()
    ERROR = auto()           
    EOF = auto()             


@dataclass(slots=True, frozen=True, kw_only=True)
class Token:
    type: TokenType
    line: int
    value: str = ""
    payload: Mapping[str, Any] = field(default_factory=dict)
    col: int = 0
