"""Fasting disciplines. One module per tradition; ``get`` finds it by name."""

from . import orthodox, ruthenian

_RULES = {
    "orthodox": orthodox.RULE,
    "ruthenian": ruthenian.RULE,
}


def get(name):
    try:
        return _RULES[name]
    except KeyError:
        raise KeyError(f"unknown fasting rule: {name!r}") from None


def names():
    return list(_RULES)
