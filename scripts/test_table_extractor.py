from kgdata.wikipedia.config import WPDataDirConfig
from kgdata.wikipedia.datasets.html_articles import html_articles, deser_html_articles
from sm.prelude import *
from rsoup.python.table_extractor import HTMLTableExtractor
from rsoup.core import TableExtractor, ContextExtractor, Table
from scripts.config import DATA_DIR

WPDataDirConfig.init(DATA_DIR / "wikipedia/20220420")

ds = html_articles()
file = ds.get_files()[0]

articles = [deser_html_articles(x) for x in M.deserialize_byte_lines(file, 100)]

# with M.Timer().watch_and_report(
#     "py.extract", append_to_file=DATA_DIR / "benchmarks/table_extractor.csv"
# ):
#     tables = []
#     for article in articles:
#         tables += HTMLTableExtractor(
#             article.url, article.html, "html5lib"
#         ).extract_tables(auto_span=True, auto_pad=True, extract_context=True)

# print(len(tables))


extractor = TableExtractor(context_extractor=ContextExtractor())
with M.Timer().watch_and_report(
    "rust.extract", append_to_file=DATA_DIR / "benchmarks/table_extractor.csv"
):
    tables: list[Table] = []
    for article in articles:
        tables += extractor.extract(
            article.url,
            article.html,
            auto_span=True,
            auto_pad=True,
            extract_context=True,
        )

with M.Timer().watch_and_report("rust.iter"):
    print(len(tables))
    for tbl in tables[:1]:
        print(">>>>>", tbl.url)
        for cell in tbl.iter_cells():
            print(">>>", str(cell))

print(tables[0].to_json())
