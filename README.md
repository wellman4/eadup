<div align="center">
  <img src="https://raw.githubusercontent.com/wellman4/eadup/refs/heads/main/assets/logo.svg" alt="EADUP Logo" width="200">

# EADUP
> **Content over appearance — automated typesetting according to regulatory requirements (GOST and NArFU corporate standards)**
</div>

Focus on the content of your work without wasting time on WYSIWYG editors. Implementing the "document as code" principle, EADUP takes care of creating a perfectly formatted document, eliminating routine formatting and ensuring strict compliance with regulatory requirements.

## Markup Language
Documents are created in `.ead` (Educational Activity Document) — a lightweight markup language designed specifically for its native compiler.

**Key Advantages:**
* **Human-readable:** A plain text format that is easy to read and edit in any text editor without specialized software.
* **Bilingual layout:** The syntax is currently optimized for the Russian keyboard layout, minimizing the need for constant language switching when writing text and commands. Future updates will introduce a flexible architecture, allowing optimization for any language, enabling users to stay entirely within their native layout.

## Example
Create a file named `document.ead` and add the following content:

```ead
ЛИСТ ДЛЯ ЗАМЕЧАНИЙ

ВВЕДЕНИЕ

The relevance of the topic is described here.

- first list item
- second list item

ОСНОВНАЯ ЧАСТЬ

/ Technical Specifications
Paragraph text describing the system.

РИСУНОК 
\имя = Figure 1.1 - Compiler Workflow Diagram
\источник = diagram.png
ПОЯСНЕНИЯ
- 1 - Input data (.ead)
- 2 - Processing
- 3 - Output file (PDF)
```

## Usage

### CLI (Terminal)
Install the compiler via Cargo:
```bash
cargo install eadup
```
Compile a document to PDF:
```bash
eadup document.ead
```

## Author
* **Developer:** wellman4
* **Year:** 2026

## License & Trademark
* **Code:** The project's source code is distributed under the **GNU AGPLv3** license.
* **Trademark:** The name **EADUP** (including variations such as **eadup**, **Eadup**, and any other case-insensitive spellings), phonetically or visually similar names (including, but not limited to, **ADUP** and **adup**), and the logo are the intellectual property of **wellman4**. When creating derivative products (forks) or commercial services based on this code, changing the name and visual identity is mandatory. Use of the original brand for third-party projects without the owner's consent is prohibited.