#  Copyright (C) 2026 Ivan Goglenkov (wellman4)
#  This program is free software: you can redistribute it and/or modify
#  it under the terms of the GNU Affero General Public License as published by
#  the Free Software Foundation, either version 3 of the License.

from dataclasses import dataclass, field
from typing import List, Dict

@dataclass
class Node:
    pass


@dataclass
class ContainerNode(Node):
    children: List[Node] = field(default_factory=list)


@dataclass
class AttributedNode(ContainerNode):
    attributes: Dict[str, str] = field(default_factory=dict)


@dataclass
class Document(AttributedNode):
    pass


@dataclass
class StructuralElement(AttributedNode):
    name: str = ""
    display_name: str = ""


@dataclass
class Heading(ContainerNode):
    level: int = 1
    title: str = ""


@dataclass
class Paragraph(Node):
    text: str = ""


@dataclass
class ListContainer(ContainerNode):
    pass


@dataclass
class ListItem(ContainerNode):
    level: int = 1
    text: str = ""


@dataclass
class Figure(AttributedNode):
    pass


@dataclass
class Table(AttributedNode):
    pass

@dataclass
class Listing(AttributedNode):
    text: str = ""

@dataclass
class TableRow(ContainerNode):
    pass


@dataclass
class TableCell(ContainerNode):
    pass


@dataclass
class NoteBlock(ContainerNode):
    type: str = ""
