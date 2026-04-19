#  Copyright (C) 2026 Ivan Goglenkov (wellman4)
#  This program is free software: you can redistribute it and/or modify
#  it under the terms of the GNU Affero General Public License as published by
#  the Free Software Foundation, either version 3 of the License.

STRUCTURAL_ELEMENTS = {
    'ТИТУЛЬНЫЙ ЛИСТ', 'ЛИСТ ДЛЯ ЗАМЕЧАНИЙ', 'РЕФЕРАТ', 'АННОТАЦИЯ', 'ОГЛАВЛЕНИЕ',
    'СОДЕРЖАНИЕ', 'НОРМАТИВНЫЕ ССЫЛКИ', 'ОПРЕДЕЛЕНИЯ ОБОЗНАЧЕНИЯ СОКРАЩЕНИЯ', 'ВВЕДЕНИЕ',
    'ОСНОВНАЯ ЧАСТЬ', 'ЗАКЛЮЧЕНИЕ', 'ВЫВОДЫ', 'СПИСОК ИСПОЛЬЗОВАННЫХ ИСТОЧНИКОВ',
    'ПРИЛОЖЕНИЕ', 'СВЕДЕНИЯ О САМОСТОЯТЕЛЬНОСТИ ВЫПОЛНЕНИЯ РАБОТЫ',
}

CONTAINERS = {
    'РИСУНОК', 'ТАБЛИЦА', 'ПРИМЕЧАНИЯ', 'ПРИМЕРЫ'
}

CHILD_ONLY = {
    'СТРОКА', 'ГРАФА', 'ПОЯСНЕНИЯ'
}

SYNONYMS = {
    'АННОТАЦИЯ': 'РЕФЕРАТ',
    'ОГЛАВЛЕНИЕ': 'СОДЕРЖАНИЕ',
    'ВЫВОДЫ': 'ЗАКЛЮЧЕНИЕ'
}
