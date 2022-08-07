from typing import List

from tests.conftest import Webpage
from table_extractor.table_extractor import TableExtractor, ContextExtractor

extractor = TableExtractor(context_extractor=ContextExtractor())


def test_table_extractor(wikipages: List[Webpage]):
    page = [page for page in wikipages if page.url.find("mountains") != -1][0]
    tables = extractor.extract(
        page.url,
        page.html,
        auto_span=True,
        auto_pad=True,
        extract_context=False,
    )

    print(tables)
