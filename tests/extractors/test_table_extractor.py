import pytest
from typing import List

from tests.conftest import Webpage
from rsoup.core import TableExtractor, ContextExtractor
from tabulate import tabulate


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
    assert len(tables) == 3
    assert tables[0].to_list() == [
        [
            "",
            "This article possibly contains original research. Please improve it by verifying the claims made and adding inline citations. Statements consisting only of original research should be removed. (September 2021) (Learn how and when to remove this template message)",
        ]
    ]
    assert tables[1].to_list() == [
        ["Map all coordinates using: OpenStreetMap"],
        ["Download coordinates as: KML"],
    ]
    # nrows, ncols = tables[2].shape()
    # print(tables[2].get_cell(1, ncols - 1).to_dict())
    # print(tables[2].get_cell(1, 0).to_dict())
    assert tables[2].to_list()[:4] == [
        [
            "Rank\n[dp 1]",
            "Mountain name(s)",
            "Height\n(rounded)\n[dp 2]",
            "Height\n(rounded)\n[dp 2]",
            "Prominence\n(rounded)\n[dp 3]",
            "Prominence\n(rounded)\n[dp 3]",
            "Range",
            "Coordinates\n[dp 4]",
            "Parent mountain\n[dp 5]",
            "Ascents before\n2004[dp 6]",
            "Ascents before\n2004[dp 6]",
            "Ascents before\n2004[dp 6]",
            "Country",
        ],
        [
            "Rank\n[dp 1]",
            "Mountain name(s)",
            "Height\n(rounded)\n[dp 2]",
            "Height\n(rounded)\n[dp 2]",
            "Prominence\n(rounded)\n[dp 3]",
            "Prominence\n(rounded)\n[dp 3]",
            "Range",
            "Coordinates\n[dp 4]",
            "Parent mountain\n[dp 5]",
            "1st",
            "successful",
            "successful",
            "Country",
        ],
        [
            "Rank\n[dp 1]",
            "Mountain name(s)",
            "m",
            "ft",
            "m",
            "ft",
            "Range",
            "Coordinates\n[dp 4]",
            "Parent mountain\n[dp 5]",
            "1st",
            "y",
            "n",
            "Country",
        ],
        [
            "1",
            "Mount Everest\nSagarmatha\nChomolungma",
            "8,848",
            "29,029\n[dp 7]",
            "8,848",
            "29,029",
            "Mahalangur Himalaya",
            "27°59′17″N 86°55′31″E\ufeff / \ufeff27.9881°N 86.9253°E\ufeff / 27.9881; 86.9253\ufeff (1. Mount Everest / Sagarmatha / Chomolungma / Zhumulangma (8848 m))",
            "—",
            "1953",
            "145",
            "121",
            "Nepal\nChina",
        ],
    ]
