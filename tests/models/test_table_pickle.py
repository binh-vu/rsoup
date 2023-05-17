from pathlib import Path
import pickle
from typing import List
import pytest
from rsoup.core import ContextExtractor, Table, TableExtractor, ContentHierarchy


@pytest.fixture
def tables(resource_dir: Path) -> List[Table]:
    extractor = TableExtractor(context_extractor=ContextExtractor())
    return extractor.extract(
        "http://example.org/table_span.html",
        (resource_dir / "table_span.html").read_text(),
        auto_span=False,
        auto_pad=False,
        extract_context=True,
    )


def test_pickle(tables: List[Table]):
    for t in tables:
        tprime: Table = pickle.loads(pickle.dumps(t))
        assert t.to_list() == tprime.to_list()
        assert t.to_dict() == tprime.to_dict()

        for c in t.context:
            cprime: ContentHierarchy = pickle.loads(pickle.dumps(c))
            assert c.to_dict() == cprime.to_dict()
