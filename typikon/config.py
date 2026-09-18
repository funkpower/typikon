"""Where the default jurisdiction is remembered.

Choosing a reckoning is a once-in-a-lifetime decision for most people, so it
should not be a flag typed every morning. The first run writes the choice down;
every run after reads it. An explicit flag always wins over the file.
"""

import os

try:
    import tomllib
except ModuleNotFoundError:            # Python < 3.11
    tomllib = None

__all__ = ["path", "read", "write", "default_jurisdiction"]

ENV_VAR = "TYPIKON_JURISDICTION"


def path():
    base = os.environ.get("XDG_CONFIG_HOME") or os.path.expanduser("~/.config")
    return os.path.join(base, "typikon", "config.toml")


def read():
    p = path()
    if not os.path.exists(p) or tomllib is None:
        return {}
    try:
        with open(p, "rb") as fh:
            return tomllib.load(fh)
    except (OSError, ValueError):
        return {}


def write(jurisdiction):
    p = path()
    os.makedirs(os.path.dirname(p), exist_ok=True)
    with open(p, "w", encoding="utf-8") as fh:
        fh.write("# Which reckoning typikon uses when no --jurisdiction is given.\n"
                 "# Run `typikon --list` to see every accepted value.\n"
                 f'jurisdiction = "{jurisdiction}"\n')
    return p


def default_jurisdiction():
    """The environment beats the config file; both beat the built-in default."""
    env = os.environ.get(ENV_VAR)
    if env:
        return env, "environment"
    conf = read().get("jurisdiction")
    if conf:
        return conf, "config"
    return "new", "default"
