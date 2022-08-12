from pathlib import Path
from rsoup.fetch_tables import default_fetch


def fix_url(html: str):
    host = "https://en.wikipedia.org"
    html = html.replace("//upload.wikimedia.org", "https://upload.wikimedia.org")
    html = html.replace('href="//', 'href="https://')
    html = html.replace('href="/', f'href="{host}/')
    html = html.replace('src="//', f'src="https://')
    html = html.replace('srcset="//', f'srcset="https://')
    html = html.replace('src="/', f'src="{host}/')

    return html


html = default_fetch("https://en.wikipedia.org/wiki/List_of_highest_mountains_on_Earth")
html = fix_url(html)

ddir = Path(__file__).parent / "data"
(ddir / "List_of_highest_mountains_on_Earth.html").write_text(html)
