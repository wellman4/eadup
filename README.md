<div align="center">
  <img src="https://raw.githubusercontent.com/wellman4/eadup/refs/heads/main/assets/logo.svg" alt="EADUP Logo" width="200">

# EADUP
> **Content over appearance — automated typesetting designed for compliance with regulatory requirements (GOST and NArFU institutional standards)**
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
* **Trademark:** The names **EADUP**, **eadup**, and the associated logo are the intellectual property of **wellman4**.
  * **Branding:** While the AGPLv3 license allows you to modify the code, it does not grant permission to use the project's trademarks or brand identity for derivative works. 
  * **Forks:** Any derivative products (forks) or services based on this code must be rebranded. Use of the name **EADUP** or any other names that imply an official connection to the original project is prohibited.

## Disclaimer
This project is an independent open-source development. It is not officially endorsed by, affiliated with, or supported by the Northern (Arctic) Federal University (NArFU). 

The references to **GOST** and **NArFU institutional standards** are provided solely for the purpose of identifying the formatting specifications that this tool aims to implement. The developer does not guarantee absolute compliance with these standards and is not responsible for any academic or legal consequences resulting from the use of this software.