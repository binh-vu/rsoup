import requests
from typing import Callable, Optional
from rsoup.python.table_extractor import HTMLTableExtractor


def fetch_tables(
    url: str,
    auto_span: bool = True,
    auto_pad: bool = True,
    fetch: Optional[Callable[[str], str]] = None,
):
    """Fetch tables from a webpage"""
    if fetch is None:
        fetch = default_fetch
    html = fetch(url)
    return HTMLTableExtractor(url, html).extract_tables(auto_span, auto_pad)


def default_fetch(url: str):
    resp = requests.get(url)
    assert resp.status_code == 200
    return resp.text
