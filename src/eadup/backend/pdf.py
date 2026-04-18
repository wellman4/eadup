#  Copyright (C) 2026 Ivan Goglenkov
#  This program is free software: you can redistribute it and/or modify
#  it under the terms of the GNU Affero General Public License as published by
#  the Free Software Foundation, either version 3 of the License.

import io
import os
import re
import csv
from reportlab.platypus import SimpleDocTemplate, Paragraph, PageBreak, Image, KeepTogether, Spacer, LongTable, TableStyle, Flowable, Table
from reportlab.lib import colors
from reportlab.lib.pagesizes import A4
from reportlab.lib.units import mm
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.enums import TA_LEFT, TA_JUSTIFY, TA_CENTER
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus.tableofcontents import TableOfContents, drawPageNumbers
from ast import literal_eval

from .. import ast as document_ast

RE_PREPOSITIONS = re.compile(r'\b(в|во|без|до|из|к|ко|на|по|о|от|перед|при|через|с|со|у|за|над|под|про|и|а|но|да|или)\b\s+', re.IGNORECASE)


class CustomTOC(TableOfContents):
    def wrap(self, availWidth, availHeight):

        def drawTOCEntryEnd(canvas, kind, label):
            label = label.split(',')
            page, level, key = int(label[0]), int(label[1]), literal_eval(label[2])
            style = self.getLevelStyle(level)
            
            if self.dotsMinLevel >= 0 and level >= self.dotsMinLevel:
                dot = '.' 
            else:
                dot = ''
                
            if self.formatter: 
                page = self.formatter(page)
            
            drawPageNumbers(canvas, style, [(page, key)], availWidth, availHeight, dot)

        self.canv.setNamedCB('drawTOCEntryEnd', drawTOCEntryEnd)

        if len(self._lastEntries) == 0:
            _tempEntries = [(0, 'Placeholder for table of contents', 0, None)]
        else:
            _tempEntries = self._lastEntries

        tableData = []
        for (level, text, pageNum, key) in _tempEntries:
            style = self.getLevelStyle(level)
            if key:
                text = '<a href="#%s">%s</a>' % (key, text)
                keyVal = repr(key).replace(',', '\\x2c').replace('"', '\\x2c')
            else:
                keyVal = None
            
            para = Paragraph('%s<onDraw name="drawTOCEntryEnd" label="%d,%d,%s"/>' % (text, pageNum, level, keyVal), style)
            
            if style.spaceBefore:
                tableData.append([Spacer(1, style.spaceBefore),])
            tableData.append([para,])

        self._table = Table(tableData, colWidths=(availWidth,), style=self.tableStyle)
        self.width, self.height = self._table.wrapOn(self.canv, availWidth, availHeight)
        return (self.width, self.height)


class PDFBackend:
    def __init__(self, root_dir=None):
        self.root_dir = root_dir or "."
        self.styles = {}
        self.font_regular = 'PTAstraSerif'
        self.font_bold = 'PTAstraSerif-Bold'
        self.heading_counters = [0, 0, 0, 0]
        self.appendix_count = 0
        self.start_numbering_page = 9999
        self.toc_data = []
        self._setup_fonts()

    def _setup_fonts(self):
        font_dir = os.path.join(os.path.dirname(__file__), 'fonts') 
        fonts = [
            (self.font_regular, 'PT-Astra-Serif/pt-astra-serif_regular.ttf'),
            (self.font_bold, 'PT-Astra-Serif/pt-astra-serif_bold.ttf'),
            (self.font_regular + '-Italic', 'PT-Astra-Serif/pt-astra-serif_italic.ttf'),
            (self.font_regular + '-BoldItalic', 'PT-Astra-Serif/pt-astra-serif_bold-italic.ttf'),
            ('PTMono', 'PT-Mono/pt-mono_regular.ttf'),
        ]
        for name, file in fonts:
            path = os.path.join(font_dir, file)
            if os.path.exists(path):
                pdfmetrics.registerFont(TTFont(name, path))

    def _get_numbering(self, level):
        """Инкремент счетчиков и возврат строки номера '1.1.1 '."""
        idx = level - 1
        self.heading_counters[idx] += 1
        for i in range(level, 4):
            self.heading_counters[i] = 0
        active = [str(n) for n in self.heading_counters[:level]]
        return ".".join(active) + " "
    
    def _reset_counters(self):
        """Полное обнуление состояния перед генерацией чистовика."""
        self.appendix_count = 0
        self.fig_count = 0
        self.table_count = 0
        
        self.heading_counters = [0] * 4 
        
        self.start_numbering_page = 999 
        
        if hasattr(self, '_pending_number'):
            self._pending_number = None


    def _setup_styles(self, doc_attrs):
        self.styles['Normal'] = ParagraphStyle(
            'Normal',
            fontName=self.font_regular,
            fontSize=self.fs,
            leading=self.fs * 1.5,
            alignment=TA_JUSTIFY,
            hyphenationLang='ru_RU',
            splitLongWords=1,
            uriWasteReduce=0.5,
            firstLineIndent=self.indent,
            spaceBefore=0,
            spaceAfter=0
        )

        self.styles['NoteText'] = ParagraphStyle(
            'NoteText',
            parent=self.styles['Normal'],
            fontSize=self.small_fs,
            leading=self.small_fs * 1.5,
            keepWithNext=True                
        )

        self.styles['Structural'] = ParagraphStyle(
            'Structural',
            parent=self.styles['Normal'],
            alignment=TA_CENTER,
            firstLineIndent=0,
            spaceBefore=0,
            spaceAfter=12,
            textTransform='uppercase'
        )

        self.styles['Heading1'] = ParagraphStyle(
            'Heading1',
            parent=self.styles['Normal'],
            fontName=self.font_bold,
            alignment=TA_JUSTIFY,
            leftIndent=self.indent,
            rightIndent=10 * mm,
            firstLineIndent=0,
            spaceAfter=12,
            textTransform='uppercase',
            hyphenation=False,   
            splitLongWords=False,
        )

        for i in range(2, 5):
            self.styles[f'Heading{i}'] = ParagraphStyle(
                f'Heading{i}',
                parent=self.styles['Normal'],
                spaceBefore=12,
                spaceAfter=12,
                hyphenation=False,   
                splitLongWords=False,
                keepWithNext=True
            )
        
        self.styles['ListLevel2'] = ParagraphStyle(
            'ListLevel2',
            parent=self.styles['Normal'],
            firstLineIndent=25 * mm, 
        )

        self.styles['AppendixHeader'] = ParagraphStyle(
            'AppendixHeader',
            parent=self.styles['Normal'],
            fontName=self.font_regular,
            fontSize=self.fs,
            leading=self.fs,
            alignment=TA_CENTER,
            spaceAfter=0,
            firstLineIndent=0
        )

        self.styles['FigureCaption'] = ParagraphStyle(
            'FigureCaption',
            parent=self.styles['Normal'],
            fontSize=self.fs,             
            leading=self.fs * 1.5,        
            alignment=TA_CENTER,     
            firstLineIndent=0,       
            spaceBefore=6,           
            spaceAfter=12            
        )

        self.styles['FigureLegend'] = ParagraphStyle(
            'FigureLegend',
            parent=self.styles['Normal'],
            fontSize=self.small_fs,       
            leading=self.small_fs,        
            alignment=TA_CENTER,     
            firstLineIndent=0,
            spaceBefore=3,           
            spaceAfter=0             
        )

        self.styles['TableName'] = ParagraphStyle(
            'TableName',
            parent=self.styles['Normal'],
            firstLineIndent=0,
            spaceBefore=0,
            spaceAfter=0,
            keepWithNext=True
        )

        self.styles['TableCellHeader'] = ParagraphStyle(
            'TableCellHeader',
            parent=self.styles['Normal'],
            fontSize=self.small_fs,
            leading=self.small_fs,  
            alignment=TA_CENTER, 
            spaceBefore=0,
            spaceAfter=0,
            firstLineIndent=0
        )

        self.styles['TableCellText'] = ParagraphStyle(
            'TableCellText',
            parent=self.styles['Normal'],
            fontSize=self.small_fs,
            leading=self.small_fs,
            alignment=TA_LEFT,
            firstLineIndent=0
        )

        self.styles['TableCellNum'] = ParagraphStyle(
            'TableCellNum',
            parent=self.styles['Normal'],
            fontSize=self.small_fs,
            leading=self.small_fs,
            alignment=TA_CENTER,
            firstLineIndent=0
        )

        self.styles['TableNote'] = ParagraphStyle(
            'TableNote',
            parent=self.styles['NoteText'],
            leading=self.small_fs,
            keepWithNext=False
        )

        self.styles['TOCBase'] = ParagraphStyle(
            'TOCBase',
            parent=self.styles['Normal'],
            fontSize=self.fs,
            leading=self.fs * 1.5,
            spaceBefore=0,
            spaceAfter=0,
            rightIndent=10 * mm,
            firstLineIndent=0,
            alignment=TA_LEFT,
        )

        self.styles['TOC0'] = ParagraphStyle('TOC0', parent=self.styles['TOCBase'], leftIndent=0)
        self.styles['TOC1'] = ParagraphStyle('TOC1', parent=self.styles['TOCBase'], leftIndent=5 * mm) 
        self.styles['TOC2'] = ParagraphStyle('TOC2', parent=self.styles['TOCBase'], leftIndent=12.5 * mm)
        self.styles['TOC3'] = ParagraphStyle('TOC3', parent=self.styles['TOCBase'], leftIndent=22 * mm) 
    
    def _add_page_break(self, story):
        if story and not isinstance(story[-1], PageBreak):
            story.append(PageBreak())

    def _draw_footer(self, canvas, doc):
        page_num = canvas.getPageNumber()
        
        if page_num < self.start_numbering_page:
            return

        canvas.saveState()
        canvas.setFont(self.font_regular, self.fs)
        
        center_x = self.left_margin + (self.available_width / 2.0)
        y_position = self.bottom_margin / 2
        
        canvas.drawCentredString(center_x, y_position, str(page_num))
        canvas.restoreState()

    def generate(self, ast_root: document_ast.Document):
        attrs = ast_root.attributes

        self.fs = int(attrs.get('основной_размер_шрифта', 14))
        self.small_fs = int(attrs.get('малый_размер_шрифта', self.fs - 2))
        self.indent = 12.5 * mm
        self.left_margin = float(attrs.get('левое_поле', 25)) * mm
        self.right_margin = float(attrs.get('правое_поле', 15)) * mm
        self.top_margin = float(attrs.get('верхнее_поле', 20)) * mm
        self.bottom_margin = float(attrs.get('нижнее_поле', 20)) * mm
        self.available_width = A4[0] - self.left_margin - self.right_margin

        self._setup_styles(attrs)

        final_buffer = io.BytesIO()
        doc = SimpleDocTemplate(
            final_buffer,
            pagesize=A4,
            topMargin=self.top_margin,
            bottomMargin=self.bottom_margin,
            leftMargin=self.left_margin,
            rightMargin=self.right_margin,
            title=attrs.get('название'),
        )

        self._reset_counters()
        story = []
        self._build_story(ast_root, story, doc)
        doc.afterFlowable = lambda flowable: self.my_after_flowable(doc, flowable)
        doc.multiBuild(
            story,
            onFirstPage=self._draw_footer,
            onLaterPages=self._draw_footer
        )
        
        return final_buffer.getvalue()

    def _build_story(self, node, story, doc, context=None):
        """Рекурсивный перевод AST в ReportLab Flowables с контекстной обработкой."""

        if isinstance(node, document_ast.StructuralElement):
            if story:
                self._add_page_break(story)

            name_upper = node.name.upper()
            text_to_print = getattr(node, 'display_name', node.name).upper()

            if name_upper == 'ПРИЛОЖЕНИЕ':
                letter = self._get_alphabet_char(self.appendix_count).upper()
                self.appendix_count += 1
                
                app_type = node.attributes.get('тип', '').strip()
                app_name = node.attributes.get('имя', '').strip()

                toc_text = f"Приложение {letter}"
                if app_type: toc_text += f" ({app_type})"
                if app_name: toc_text += f" {app_name.capitalize()}"

                p1 = Paragraph(f"ПРИЛОЖЕНИЕ {letter}", self.styles['AppendixHeader'])
                p1._is_appendix_start = True
                p1._toc_text = toc_text
                story.append(p1)

                if app_type:
                    story.append(Paragraph(f"({app_type})", self.styles['AppendixHeader']))
                
                if app_name:
                    story.append(Paragraph(app_name, self.styles['AppendixHeader']))
                
                story.append(Spacer(1, 12))
                
                self._traverse_children(node, story, doc, context=f"ПРИЛОЖЕНИЕ_{letter}")
                return

            if name_upper == 'СОДЕРЖАНИЕ':
                def save_page(canvas):
                    self.start_numbering_page = canvas.getPageNumber()
                story.append(NumberingStartMarker(save_page))

                story.append(Paragraph(text_to_print, self.styles['Structural']))
    
                toc = CustomTOC()
                toc.dotsMinLevel = 0
                toc.levelStyles = [
                    self.styles['TOC0'],
                    self.styles['TOC1'],
                    self.styles['TOC2'],
                    self.styles['TOC3'],
                ]
                toc.tableStyle = TableStyle([
                    ('VALIGN', (0,0), (-1,-1), 'TOP'),
                    ('TOPPADDING', (0,0), (-1,-1), 0),
                    ('BOTTOMPADDING', (0,0), (-1,-1), 0),
                    ('LEFTPADDING', (0,0), (-1,-1), 0),
                    ('RIGHTPADDING', (0,0), (-1,-1), 0),
                ])
                story.append(toc)
                
                return
                
            if name_upper == 'ОСНОВНАЯ ЧАСТЬ':
                self._traverse_children(node, story, doc, context=name_upper)
                return

            if name_upper == 'ТИТУЛЬНЫЙ ЛИСТ':
                self._render_title_page(node, story)
                return 
            if name_upper == 'СВЕДЕНИЯ О САМОСТОЯТЕЛЬНОСТИ ВЫПОЛНЕНИЯ РАБОТЫ':
                self._render_independence_page(node, story)
                return

            if name_upper == 'ОПРЕДЕЛЕНИЯ ОБОЗНАЧЕНИЯ СОКРАЩЕНИЯ':
                text_to_print = 'ОПРЕДЕЛЕНИЯ, ОБОЗНАЧЕНИЯ И СОКРАЩЕНИЯ'
            
            story.append(Paragraph(text_to_print, self.styles['Structural']))

            if name_upper == 'НОРМАТИВНЫЕ ССЫЛКИ':
                story.append(Paragraph(
                    "В настоящей текстовом документе использованы ссылки на следующие нормативные документы:", 
                    self.styles['Normal']
                ))
            elif name_upper == 'ОПРЕДЕЛЕНИЯ ОБОЗНАЧЕНИЯ СОКРАЩЕНИЯ':
                story.append(Paragraph(
                    "В настоящем текстовом документе применяются следующие определения, обозначения и сокращения:", 
                    self.styles['Normal']
                ))

            self._traverse_children(node, story, doc, context=name_upper)
            return

        elif isinstance(node, document_ast.ListContainer):
            special_sections = {
                'НОРМАТИВНЫЕ ССЫЛКИ', 
                'ОПРЕДЕЛЕНИЯ ОБОЗНАЧЕНИЯ СОКРАЩЕНИЯ', 
                'СПИСОК ИСПОЛЬЗОВАННЫХ ИСТОЧНИКОВ'
            }

            if context in special_sections:
                for i, item in enumerate(node.children, 1):
                    if isinstance(item, document_ast.ListItem):
                        text = item.text.strip()
                        
                        if context == 'СПИСОК ИСПОЛЬЗОВАННЫХ ИСТОЧНИКОВ':
                            text = f"{i} {text}"
                        
                        if text:
                            story.append(Paragraph(text, self.styles['Normal']))
            else:
                self._render_list(node, story, doc)
            return

        elif isinstance(node, document_ast.Heading):
            if node.level == 1:
                self._add_page_break(story)
            
            number = self._get_numbering(node.level)
            style = self.styles[f'Heading{min(node.level, 4)}']
            
            clean_title = node.title.strip().rstrip('.,;:')
            
            if clean_title:
                processed_title = RE_PREPOSITIONS.sub(f"\\1\u00A0", clean_title)
                
                full_title = f"{number} {processed_title}"
                story.append(Paragraph(full_title, style))
            else:
                self._pending_number = number 
            
            self._traverse_children(node, story, doc, context=context)
            return

        elif isinstance(node, document_ast.Paragraph):
            text = node.text.strip()
            if text:
                if hasattr(self, '_pending_number') and self._pending_number:
                    text = f"{self._pending_number}{text}"
                    self._pending_number = None
                
                story.append(Paragraph(text, self.styles['Normal']))
            return

        elif isinstance(node, document_ast.Document):
            self._traverse_children(node, story, doc, context=context)
            return

        elif isinstance(node, document_ast.NoteBlock):
            items = []
            for child in node.children:
                if isinstance(child, document_ast.ListContainer):
                    items.extend([i for i in child.children if isinstance(i, document_ast.ListItem)])
            
            if not items:
                return

            kind_raw = node.type.upper()
            is_multiple = len(items) > 1

            names = {
                'ПРИМЕЧАНИЯ': ('Примечание', 'Примечания'),
                'ПРИМЕРЫ': ('Пример', 'Примеры')
            }
            singular, plural = names.get(kind_raw, ('Примечание', 'Примечания'))

            if not is_multiple:
                item = items[0]
                text = item.text.strip()
                if text and not text.endswith('.'): text += '.'
                
                story.append(Paragraph(f"{singular} – {text}", self.styles['NoteText']))
            
            else:
                story.append(Paragraph(plural, self.styles['NoteText']))
                
                for i, item in enumerate(items, 1):
                    text = item.text.strip()
                    if text and not text.endswith('.'): text += '.'
                    
                    story.append(Paragraph(f"{i} {text}", self.styles['NoteText']))
            
            return

        elif isinstance(node, document_ast.Figure):
            img_file = node.attributes.get('источник')
            caption = node.attributes.get('имя')
            parts = []

            if img_file:
                img_file = img_file.strip().strip("'\"")
                path = os.path.join(self.root_dir, img_file)
                
                if os.path.exists(path):
                    img = Image(path)
                    
                    target_w = self.available_width
                    
                    ratio = target_w / img.drawWidth
                    img.drawWidth = target_w
                    img.drawHeight *= ratio
                    
                    max_h = A4[1] - self.top_margin - self.bottom_margin - 40*mm
                    if img.drawHeight > max_h:
                        h_ratio = max_h / img.drawHeight
                        img.drawHeight = max_h
                        img.drawWidth *= h_ratio
                    
                    img.hAlign = 'CENTER'
                    
                    parts.append(Spacer(1, 12)) 
                    parts.append(img)

            legend_text = ""
            notes_story = [] 

            if hasattr(node, 'children'):
                for child in node.children:
                    if isinstance(child, document_ast.NoteBlock):
                        node_kind = getattr(child, 'type', '').upper()
                        if node_kind == 'ПОЯСНЕНИЯ':
                            texts = []
                            for c in child.children:
                                if isinstance(c, document_ast.ListContainer):
                                    texts.extend([i.text.strip() for i in c.children])
                            legend_text = "; ".join(texts)
                        else:
                            self._build_story(child, notes_story, doc)

            if legend_text:
                parts.append(Paragraph(legend_text, self.styles['FigureLegend']))

            if caption:
                full_caption = caption.rstrip('.,;:')
                parts.append(Paragraph(full_caption, self.styles['FigureCaption']))

            if parts:
                story.append(KeepTogether(parts))
            
            if notes_story:
                story.extend(notes_story)
            
            return
        
        elif isinstance(node, document_ast.Table):
            table_name = node.attributes.get('имя', '').strip()
            source_file = node.attributes.get('источник', '').strip().strip("'\"")
            data = []
            table_styles = [
                ('GRID', (0, 0), (-1, -1), 0.5, colors.black),
                ('TOPPADDING', (0, 0), (-1, -1), 2),
                ('BOTTOMPADDING', (0, 0), (-1, -1), 4),
            ]

            if source_file:
                path = os.path.join(self.root_dir, source_file)
                if os.path.exists(path):
                    try:
                        with open(path, 'r', encoding='utf-8') as f:
                            reader = csv.reader(f)
                            for r_idx, row_list in enumerate(reader):
                                row_data = [
                                    self._prepare_table_cell(txt, r_idx, c_idx, table_styles) 
                                    for c_idx, txt in enumerate(row_list)
                                ]
                                data.append(row_data)
                    except Exception as e:
                        print(f"Ошибка чтения CSV {path}: {e}")
            
            else:
                for r_idx, row in enumerate(node.children):
                    if not isinstance(row, document_ast.TableRow): continue
                    row_data = []
                    
                    for c_idx, cell in enumerate(row.children):
                        if not isinstance(cell, document_ast.TableCell): continue

                        cell_text = "".join(child.text for child in cell.children if hasattr(child, 'text'))
                        
                        prepared_cell = self._prepare_table_cell(cell_text, r_idx, c_idx, table_styles)
                        row_data.append(prepared_cell)

                    data.append(row_data)

            if data:
                if table_name:
                    story.append(Paragraph(table_name, self.styles['TableName']))

                num_cols = len(data[0])
                col_widths = [self.available_width / num_cols] * num_cols
                
                t = LongTable(
                    data, 
                    colWidths=col_widths, 
                    repeatRows=1, 
                    splitByRow=1,
                )
                t.setStyle(TableStyle(table_styles))

                t.spaceAfter = 12

                story.append(t)

            return

    def _traverse_children(self, node, story, doc, context=None):
        if hasattr(node, 'children'):
            for child in node.children:
                self._build_story(child, story, doc, context=context)

    def _prepare_table_cell(self, cell_text, r_idx, c_idx, table_styles):
        cell_text = cell_text.strip().strip("")
        is_header = (r_idx == 0)
        is_numeric = self._is_number(cell_text)

        if is_numeric and not is_header:
            cell_text = cell_text.replace('.', ',')

        coords = (c_idx, r_idx)

        if is_header:
            table_styles.append(('ALIGN', coords, coords, 'CENTER'))
            table_styles.append(('VALIGN', coords, coords, 'TOP'))
            return Paragraph(cell_text, self.styles['TableCellHeader'])
        
        elif is_numeric:
            table_styles.append(('ALIGN', coords, coords, 'CENTER'))
            table_styles.append(('VALIGN', coords, coords, 'MIDDLE'))
            table_styles.append(('FONTNAME', coords, coords, self.font_regular))
            table_styles.append(('FONTSIZE', coords, coords, self.small_fs))
            table_styles.append(('LEADING', coords, coords, self.small_fs))
            return cell_text
        
        else:
            table_styles.append(('ALIGN', coords, coords, 'LEFT'))
            table_styles.append(('VALIGN', coords, coords, 'TOP'))
            return Paragraph(cell_text, self.styles['TableCellText'])

    def _render_title_page(self, node, story):
        attrs = node.attributes
        work_type = attrs.get('тип', 'КУРСОВАЯ РАБОТА').upper()
        
        story.append(Spacer(1, 20*mm))
        story.append(Paragraph("Министерство образования...", self.styles['Normal']))
        story.append(Spacer(1, 50*mm))
        story.append(Paragraph(work_type, self.styles['Normal']))
        story.append(Spacer(1, 10*mm))
        story.append(Paragraph(f"на тему: «»", self.styles['Normal']))

    def _render_independence_page(self, node, story):
        story.append(Paragraph("СВЕДЕНИЯ", self.styles['Normal']))
        story.append(Paragraph("о самостоятельности выполнения работы", self.styles['Normal']))
        story.append(Spacer(1, 20*mm))
        text = "Я, нижеподписавшийся, подтверждаю, что работа выполнена мною самостоятельно..."
        story.append(Paragraph(text, self.styles['Normal']))
        story.append(Spacer(1, 30*mm))
        story.append(Paragraph("____________ / ____________", self.styles['Normal']))

    def _get_alphabet_char(self, index):
        alphabet = "абвгдежзиклмнпрстуфхцшщэюя"
        return alphabet[index] if index < len(alphabet) else str(index)
    
    def _is_number(self, s: str) -> bool:
        """Проверка, является ли строка числом."""
        try:
            float(s.replace(',', '.'))
            return True
        except ValueError:
            return False

    def _smart_lower(self, text):
        text = text.strip()
        if not text:
            return ""
        
        words = text.split()
        first_word = words[0]
        
        if first_word.isupper() and len(first_word) > 1:
            return text
            
        if len(first_word) > 1 and first_word[1].isupper():
            return text

        return text[0].lower() + text[1:]

    def _fix_punctuation(self, text, target_char):
        """Принудительная установка знака препинания в конце."""
        text = text.strip()
        if not text:
            return ""
            
        while text and text[-1] in {'.', ',', ';', ':'}:
            text = text[:-1].strip()
            
        if not text: return ""

        if text[-1] in {'!', '?'}:
            return text
            
        return text + target_char

    def _render_list(self, node, story, doc, level=1, is_parent_last=True):
        """Отрисовка перечислений по ГОСТ 2.105-2019 и 7.32-2017."""
        
        items = [c for c in node.children if isinstance(c, document_ast.ListItem)]
        total_items = len(items)

        for i, item in enumerate(items):
            is_last_in_current = (i == total_items - 1)
            has_inner = any(isinstance(c, document_ast.ListContainer) for c in item.children)
            
            is_absolutely_last_here = is_last_in_current and is_parent_last and not has_inner
            
            if level == 1:
                prefix = f"{self._get_alphabet_char(i)}) " if has_inner else "- "
                current_style = self.styles['Normal']
            else:
                prefix = f"{i + 1}) "
                current_style = self.styles['ListLevel2']

            target_char = "." if is_absolutely_last_here else ";"
            
            raw_text = item.text.strip()
            if raw_text:
                processed_text = self._smart_lower(raw_text)
                final_text = self._fix_punctuation(processed_text, target_char)
                story.append(Paragraph(f"{prefix}{final_text}", current_style))

            if hasattr(item, 'children'):
                for child in item.children:
                    if isinstance(child, document_ast.ListContainer):
                        self._render_list(
                            child, 
                            story,
                            doc, 
                            level=level + 1, 
                            is_parent_last=(is_last_in_current and is_parent_last)
                        )
                    elif not isinstance(child, (document_ast.ListItem, document_ast.ListContainer)):
                        self._build_story(child, story, doc)
    
    def my_after_flowable(self, doc, flowable):
        if not isinstance(flowable, Paragraph):
            return
        
        if hasattr(flowable, '_is_appendix_start'):
            doc.notify('TOCEntry', (0, flowable._toc_text, doc.page))
            return
        
        style_name = flowable.style.name
        if style_name == 'AppendixHeader':
            return
        
        text = flowable.getPlainText().strip()

        if style_name.startswith('Heading'):
            h_level = int(style_name.replace('Heading', ''))
            toc_level = 0 if h_level == 1 else h_level - 1

            parts = text.split(maxsplit=1)
            if len(parts) > 1:
                number, title = parts[0], parts[1]
                formatted_text = f"{number} {title.capitalize()}"
            else:
                formatted_text = text.capitalize()

            doc.notify('TOCEntry', (toc_level, formatted_text, doc.page))

        elif style_name == 'Structural':
            excluded = {'ТИТУЛЬНЫЙ ЛИСТ', 'ЛИСТ ДЛЯ ЗАМЕЧАНИЙ', 'РЕФЕРАТ', 'АННОТАЦИЯ', 'СОДЕРЖАНИЕ', 'ОГЛАВЛЕНИЕ', 'ОСНОВНАЯ ЧАСТЬ'}
            if text.upper() not in excluded:
                doc.notify('TOCEntry', (0, text.capitalize(), doc.page))


class NumberingStartMarker(Flowable):
    """Невидимый элемент, который фиксирует страницу начала нумерации."""
    def __init__(self, callback):
        Flowable.__init__(self)
        self.callback = callback

    def draw(self):
        self.callback(self.canv)

    def wrap(self, *args):
        return (0, 0)
    

class TOCEntryMarker(Flowable):
    def __init__(self, level, text, toc_object):
        Flowable.__init__(self)
        self.level = level
        self.text = text
        self.toc_object = toc_object
        self.width = self.height = 0
        self._done = False

    def draw(self):
        if not self._done:
            page = self.canv.getPageNumber()
            self.toc_object.addEntry(self.level, self.text, page)
            print(f"--- Прямая запись в TOC: {self.text} на стр {page}")
            self._done = True

    def wrap(self, *args):
        return (0, 0)
