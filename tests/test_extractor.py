import pytest
from typing import List

from tests.conftest import Webpage
from rsoup.rsoup import TableExtractor, ContextExtractor


@pytest.fixture
def extractor():
    return TableExtractor(context_extractor=ContextExtractor())


def test_table_extractor(extractor: TableExtractor, wikipages: List[Webpage]):
    page = [page for page in wikipages if page.url.find("mountains") != -1][0]
    tables = extractor.extract(
        page.url,
        page.html,
        auto_span=True,
        auto_pad=True,
        extract_context=True,
    )

    print([t.to_bytes() for t in tables])
