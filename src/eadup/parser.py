#  Copyright (C) 2026 Ivan Goglenkov (wellman4)
#  This program is free software: you can redistribute it and/or modify
#  it under the terms of the GNU Affero General Public License as published by
#  the Free Software Foundation, either version 3 of the License.

from typing import Type, Iterable
from .ast import *
from .tokens import Token, TokenType

VALID_CHILDREN = {
    Document: {TokenType.STRUCTURAL, TokenType.ATTRIBUTE, TokenType.EMPTY_LINE},
    StructuralElement: {
        TokenType.PARAGRAPH, TokenType.HEADING, TokenType.FIGURE, TokenType.TABLE,
        TokenType.LIST_ITEM, TokenType.NOTE, TokenType.ATTRIBUTE, TokenType.EMPTY_LINE,
        TokenType.LISTING
    },
    Figure: {
        TokenType.ATTRIBUTE, 
        TokenType.NOTE, 
        TokenType.EMPTY_LINE
    },
    Table: {TokenType.ROW, TokenType.ATTRIBUTE, TokenType.EMPTY_LINE},
    TableRow: {TokenType.CELL},
    TableCell: {TokenType.PARAGRAPH, TokenType.LIST_ITEM, TokenType.EMPTY_LINE},
    NoteBlock: {TokenType.LIST_ITEM},
    Heading: {TokenType.PARAGRAPH, TokenType.LIST_ITEM, TokenType.FIGURE, TokenType.TABLE, TokenType.LISTING, TokenType.NOTE, TokenType.HEADING},
    Listing: {TokenType.ATTRIBUTE}
}

TOKEN_TO_NODE: Dict[TokenType, Type[Node]] = {
    TokenType.STRUCTURAL: StructuralElement,
    TokenType.HEADING: Heading,
    TokenType.TABLE: Table,
    TokenType.FIGURE: Figure,
    TokenType.ROW: TableRow,
    TokenType.CELL: TableCell,
    TokenType.NOTE: NoteBlock,
    TokenType.PARAGRAPH: Paragraph,
    TokenType.LIST_ITEM: ListItem,
    TokenType.LISTING: Listing
}


class Parser:
    def __init__(self):
        self.root = Document()
        self.stack: List[ContainerNode] = [self.root]

    def parse(self, tokens: Iterable[Token]) -> Document:
        for token in tokens:
            if token.type == TokenType.EOF: break
            
            if token.type == TokenType.EMPTY_LINE:
                self._handle_empty_line()
                continue

            if token.type == TokenType.ATTRIBUTE:
                self._handle_attribute(token)
                continue

            self._process_token(token)
        return self.root

    def _handle_empty_line(self):
        """Закрывает контейнеры, требующие явного завершения пустой строкой."""
        while len(self.stack) > 1:
            top = self.stack[-1]
            if isinstance(top, (TableCell, TableRow, Table, ListContainer, ListItem, NoteBlock, Figure)):
                self.stack.pop()
            else:
                break

    def _handle_attribute(self, token: Token):
        """Записывает атрибут в самый верхний узел стека, который их поддерживает."""
        key = token.payload.get('key')
        val = token.payload.get('text')
        
        for node in reversed(self.stack):
            if isinstance(node, AttributedNode):
                node.attributes[key] = val
                return
        self.root.attributes[key] = val

    def _process_token(self, token: Token):
        if token.type == TokenType.LIST_ITEM:
            self._handle_list_item(token)
            return

        node_class = TOKEN_TO_NODE.get(token.type)
        if not node_class: return

        new_node = self._create_node_instance(node_class, token)

        if isinstance(new_node, Heading):
            while len(self.stack) > 1:
                top = self.stack[-1]
                if isinstance(top, Heading) and top.level >= new_node.level:
                    self.stack.pop()
                elif isinstance(top, (Figure, Table, NoteBlock, ListContainer, ListItem)):
                    self.stack.pop()
                else:
                    break

        while len(self.stack) > 1 and not self._can_contain(self.stack[-1], token.type):
            self.stack.pop()

        parent = self.stack[-1]
        if isinstance(parent, ContainerNode):
            parent.children.append(new_node)

        if isinstance(new_node, ContainerNode):
            self.stack.append(new_node)

    def _handle_list_item(self, token: Token):
        level = token.payload.get('level', 1)
        text = token.payload.get('text', '')

        while len(self.stack) > 1:
            top = self.stack[-1]
            
            if isinstance(top, ListItem) and top.level >= level:
                self.stack.pop()
                continue
            
            if isinstance(top, ListContainer) and not (
                isinstance(self.stack[-2], ListItem) and self.stack[-2].level < level
            ):
                if level == 1 and isinstance(self.stack[-2], ListItem):
                    self.stack.pop()
                    continue
            
            break

        if not isinstance(self.stack[-1], (ListContainer, ListItem)):
            new_list = ListContainer()
            self.stack[-1].children.append(new_list)
            self.stack.append(new_list)

        if isinstance(self.stack[-1], ListItem) and self.stack[-1].level < level:
            sub_list = ListContainer()
            self.stack[-1].children.append(sub_list)
            self.stack.append(sub_list)

        new_item = ListItem(level=level, text=text)
        if isinstance(self.stack[-1], ListContainer):
            self.stack[-1].children.append(new_item)
            self.stack.append(new_item)

    def _create_node_instance(self, cls, token):
        p = token.payload
        if cls == Paragraph: return Paragraph(text=p.get('text', ''))
        if cls == Heading: return Heading(level=p.get('level', 1), title=p.get('title', ''))
        if cls == StructuralElement: return StructuralElement(name=p.get('name'), display_name=p.get('display_name'))
        if cls == NoteBlock: return NoteBlock(type=p.get('kind'))
        if cls == Figure: return Figure()
        if cls == TableCell:
            node = TableCell()
            text_content = p.get('text', '').strip()
            if text_content:
                node.children.append(Paragraph(text=text_content))
            return node
        if cls == Listing:
            return Listing(text=p.get('text'))
        return cls()

    def _can_contain(self, parent, child_type):
        allowed = VALID_CHILDREN.get(type(parent), set())
        return child_type in allowed

    def print_ast(self, node=None, indent=0):
        """Рекурсивная печать дерева AST."""
        if node is None:
            node = self.root

        node_name = node.__class__.__name__

        props = []
        if hasattr(node, 'name') and node.name:
            props.append(f"name='{node.name}'")
        if hasattr(node, 'text') and node.text:
            # Обрезаем длинный текст для красоты
            short_text = (node.text[:40] + '..') if len(node.text) > 40 else node.text
            props.append(f"text='{short_text}'")
        if hasattr(node, 'title') and node.title:
            props.append(f"title='{node.title}'")
        if hasattr(node, 'level') and node.level:
            props.append(f"level={node.level}")
        if hasattr(node, 'type') and node.type:
            props.append(f"type='{node.type}'")

        props_str = f"({', '.join(props)})" if props else ""

        attr_str = ""
        if hasattr(node, 'attributes') and node.attributes:
            attr_str = f" [attrs: {node.attributes}]"

        print("  " * indent + f"{node_name}{props_str}{attr_str}")

        if hasattr(node, 'children'):
            for child in node.children:
                self.print_ast(child, indent + 1)

