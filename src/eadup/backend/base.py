#  Copyright (C) 2026 Ivan Goglenkov
#  This program is free software: you can redistribute it and/or modify
#  it under the terms of the GNU Affero General Public License as published by
#  the Free Software Foundation, either version 3 of the License.

from abc import ABC, abstractmethod
from eadup.ast import Document

class BaseBackend(ABC):
    @abstractmethod
    def generate(self, ast) -> bytes | str:
        pass