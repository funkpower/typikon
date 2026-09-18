"""Tests for the Python bindings.

The one that matters is `test_matches_cli`: the bindings and the binary have to
answer identically for the same date, because they are the same engine and any
divergence means the Python layer has quietly grown its own opinion. It caught a
real one — the bindings were resolving a built-in default jurisdiction while the
CLI read the configured one, so every field downstream of that differed.

    python -m pytest tests/python -q

Needs the wheel installed (`maturin develop` or `pip install target/wheels/*.whl`)
and, for the parity test, `cargo build --release`.
"""

import datetime
import json
import re
import subprocess
from pathlib import Path

import pytest

typikon = pytest.importorskip("typikon")

BINARY = Path(__file__).resolve().parents[2] / "target" / "release" / "typikon"

# Pascha, Nativity, Theophany and an ordinary weekday: the movable cycle, the
# fixed calendar, the julian offset and the plain case.
DATES = [
    datetime.date(2026, 4, 12),
    datetime.date(2026, 12, 25),
    datetime.date(2027, 1, 7),
    datetime.date(2026, 3, 18),
]


def _snake(s: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "_", s).lower()


@pytest.mark.parametrize("date", DATES, ids=str)
def test_matches_cli(date: datetime.date) -> None:
    if not BINARY.exists():
        pytest.skip(f"{BINARY} not built")
    out = subprocess.run(
        [str(BINARY), "--json", "--date", date.isoformat()],
        capture_output=True, text=True, check=True,
    ).stdout
    cli = {_snake(k): v for k, v in json.loads(out).items()}
    assert typikon.describe(date).to_dict() == cli


def test_pascha_is_its_own_zero() -> None:
    for year in (2024, 2025, 2026, 2030):
        p = typikon.pascha(year)
        assert typikon.describe(p).days_to_pascha == 0
        assert p.year == year


def test_week_length_and_order() -> None:
    start = datetime.date(2026, 4, 10)
    days = typikon.week(start, 5)
    assert len(days) == 5
    assert [d.date for d in days] == [
        start + datetime.timedelta(days=i) for i in range(5)
    ]


def test_week_rejects_nonsense() -> None:
    with pytest.raises(ValueError):
        typikon.week(datetime.date(2026, 4, 10), 0)


def test_unknown_jurisdiction_raises() -> None:
    with pytest.raises(ValueError, match="unknown jurisdiction"):
        typikon.describe(datetime.date(2026, 4, 12), jurisdiction="narnia")


def test_jurisdiction_actually_changes_the_answer() -> None:
    # Old and new calendar disagree about which church date a civil date is, so
    # on 25 December the new calendar keeps the Nativity and the old calendar is
    # still thirteen days short of it. julian_offset is not the witness to reach
    # for: it is the gap between the calendars, so it reads 13 either way.
    old = typikon.describe(datetime.date(2026, 12, 25), jurisdiction="old")
    new = typikon.describe(datetime.date(2026, 12, 25), jurisdiction="new")
    assert old.church_date != new.church_date
    assert old.title != new.title
    assert "Nativity" in (new.feast or "")


def test_every_jurisdiction_resolves() -> None:
    keys = [j["key"] for j in typikon.jurisdictions()]
    assert keys, "no jurisdictions listed"
    for key in keys:
        assert typikon.describe(datetime.date(2026, 4, 12), jurisdiction=key)


def test_day_is_immutable() -> None:
    day = typikon.describe(datetime.date(2026, 4, 12))
    with pytest.raises(AttributeError):
        day.feast = "something else"  # type: ignore[misc]


def test_accepts_datetime_as_well_as_date() -> None:
    # to_date reads attributes rather than the datetime C API, so anything with
    # year/month/day works. Worth pinning: it is a consequence of the abi3 build,
    # not a decision, and it would be easy to lose by accident.
    d = datetime.date(2026, 4, 12)
    assert typikon.describe(datetime.datetime(2026, 4, 12, 13, 30)).to_dict() == \
        typikon.describe(d).to_dict()


def test_to_dict_is_json_serialisable() -> None:
    assert json.dumps(typikon.describe(datetime.date(2026, 4, 12)).to_dict())
