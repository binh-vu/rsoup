from dataclasses import dataclass
from typing import Dict, List, Optional

class Document:
    def __init__(self, url: str, doc: str): ...

class ContextExtractor:
    def __init__(
        self,
        ignored_tags: Optional[List[str]] = None,
        discard_tags: Optional[List[str]] = None,
        same_content_level_elements: Optional[List[str]] = None,
        header_elements: Optional[List[str]] = None,
        only_keep_inline_tags: bool = True,
    ): ...

class TableExtractor:
    def __init__(
        self,
        context_extractor: ContextExtractor,
        ignored_tags: Optional[List[str]] = None,
        discard_tags: Optional[List[str]] = None,
        only_keep_inline_tags: bool = True,
    ) -> None: ...
    def extract(
        self,
        url: str,
        doc: str,
        auto_span: bool = True,
        auto_pad: bool = True,
        extract_context: bool = True,
    ) -> List[Table]: ...

class Table:
    id: str
    url: str
    caption: str

    @property
    def attrs(self) -> Dict[str, str]: ...
    @property
    def context(self) -> List[ContentHierarchy]: ...
    @property
    def rows(self) -> List[Row]: ...
    def to_bytes(self) -> bytes: ...
    @staticmethod
    def from_bytes(dat: bytes) -> Table: ...
    def to_dict(self) -> dict: ...

class Row:
    @property
    def cells(self) -> List[Cell]: ...
    @property
    def attrs(self) -> Dict[str, str]: ...
    def to_dict(self) -> dict: ...

class Cell:
    is_header: bool
    rowspan: int
    colspan: int
    value: RichText
    html: str

    @property
    def attrs(self) -> Dict[str, str]: ...
    def to_dict(self) -> dict: ...

class ContentHierarchy:
    level: int
    heading: RichText

    @property
    def content_before(self) -> List[RichText]: ...
    @property
    def content_after(self) -> List[RichText]: ...
    def to_dict(self) -> dict: ...

class RichText:
    @property
    def text(self) -> str: ...
    def to_dict(self) -> dict: ...
