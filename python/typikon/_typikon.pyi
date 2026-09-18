import datetime
from typing import Any, Final

__version__: Final[str]

class Day:
    """One day, fully reckoned. Immutable."""

    @property
    def date(self) -> datetime.date: ...
    @property
    def iso(self) -> str: ...
    @property
    def weekday(self) -> str: ...
    @property
    def jurisdiction(self) -> str: ...
    @property
    def calendar(self) -> str: ...
    @property
    def church_date(self) -> str:
        """The ecclesiastical date, as "25 December"."""
    @property
    def season(self) -> str | None: ...
    @property
    def cycle(self) -> str | None: ...
    @property
    def feast(self) -> str | None: ...
    @property
    def rank(self) -> str | None: ...
    @property
    def sign(self) -> str | None:
        """The mark the liturgical books print beside a day of this rank."""
    @property
    def tradition(self) -> str:
        """Greek or Slavic usage, as this jurisdiction reads the Typikon."""
    @property
    def title(self) -> str:
        """The feast if there is one, else the principal commemoration."""
    @property
    def commemorations(self) -> list[str]: ...
    @property
    def fast(self) -> str: ...
    @property
    def fast_label(self) -> str: ...
    @property
    def emoji(self) -> str: ...
    @property
    def reason(self) -> str: ...
    @property
    def obligation(self) -> str | None:
        """``"obligatory"``, ``"customary"``, or ``None``."""
    @property
    def binds(self) -> str: ...
    @property
    def pascha(self) -> datetime.date: ...
    @property
    def days_to_pascha(self) -> int: ...
    @property
    def julian_offset(self) -> int: ...
    def is_obligatory(self) -> bool: ...
    def to_dict(self) -> dict[str, Any]:
        """The same fields as ``typikon --json``."""

def describe(
    date: datetime.date,
    jurisdiction: str | None = None,
    tradition: str | None = None,
    typikon: bool | None = None,
) -> Day: ...
def today(
    jurisdiction: str | None = None,
    tradition: str | None = None,
    typikon: bool | None = None,
) -> Day: ...
def week(
    start: datetime.date,
    days: int = 7,
    jurisdiction: str | None = None,
    tradition: str | None = None,
    typikon: bool | None = None,
) -> list[Day]: ...
def pascha(year: int, jurisdiction: str | None = None) -> datetime.date: ...
def jurisdictions() -> list[dict[str, str]]: ...
