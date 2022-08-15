from dataclasses import dataclass
from pathlib import Path
from typing import List

import pytest


@dataclass
class Webpage:
    url: str
    html: str


@pytest.fixture(scope="session")
def resource_dir() -> Path:
    """Get the testing directory"""
    return Path(__file__).absolute().parent / "resources"


@pytest.fixture(scope="session")
def wikipages(resource_dir: Path) -> List[Webpage]:
    lst = []
    for file in (resource_dir / "wikipedia").glob("*.html"):
        lst.append(
            Webpage(
                url=f"https://en.wikipedia.org/wiki/{file.stem}", html=file.read_text()
            )
        )
    return lst
