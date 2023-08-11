from __future__ import annotations

from pathlib import Path
import pickle
import pytest
from rsoup.core import ContextExtractor, RichText, Table, TableExtractor, ContentHierarchy


@pytest.fixture
def tables(resource_dir: Path) -> list[Table]:
    extractor = TableExtractor(context_extractor=ContextExtractor())
    return extractor.extract(
        "http://example.org/table_span.html",
        (resource_dir / "table_span.html").read_text(),
        auto_span=False,
        auto_pad=False,
        extract_context=True,
    )


def test_content_hierarchy_pickle(tables: list[Table]):
    for t in tables:
        tprime: Table = pickle.loads(pickle.dumps(t))
        assert t.to_list() == tprime.to_list()
        assert t.to_dict() == tprime.to_dict()

        for c in t.context:
            cprime: ContentHierarchy = pickle.loads(pickle.dumps(c))
            assert c.to_dict() == cprime.to_dict()


def test_rich_text_pickle(tables: list[Table]):
    for t in tables:
        for row in t.rows:
            for cell in row.cells:
                cell_prime: RichText = pickle.loads(pickle.dumps(cell.value))
                assert cell.value.to_dict() == cell_prime.to_dict()