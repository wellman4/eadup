<div align="center">
  <img src="assets/logo.png" alt="Логотип" width="200">
</div> 

# EADUP

**Компилятор документов учебной деятельности обучающихся в соответствии с СТО САФУ**

Инструмент для автоматической генерации и оформления учебных документов согласно требованиям стандарта организации «Северного (Арктического) федерального университета имени М. В. Ломоносова».

.ead (Educational Activity Document) — облегченный язык разметки, разрабатываемый параллельно с нативным компилятором. Ключевые особенности:
*   **Простота (Human-readable):** Текстовый формат, который легко читать и редактировать в текстовом редакторе.
*   **Стандартизация (Compliance):** Компилятор автоматически генерирует документы, строго соответствующие требованиям ГОСТ/СТО.

## Автор / Author
* **Разработчик:** Иван Гогленков (wellman4)
* **Год:** 2026

## Установка / Installation
```bash
# 1. Клонирование репозитория / Clone the repository
git clone git@github.com:wellman4/eadup.git
cd eadup

# 2. Создание и активация виртуального окружения / Create and activate venv
python -m venv venv
# Windows:
venv\Scripts\activate
# Linux/macOS:
source venv/bin/activate

# 3. Установка пакета и зависимостей / Install package and dependencies
pip install --upgrade pip
pip install -e .
```

## Использование / Usage
```bash
eadup
```
