#!/usr/bin/env python3
"""Run every console example in the README and diff it against the real output.

A README whose examples have quietly stopped being true is worse than one with
no examples in it, and the output of this program changes shape often enough
that hand-checking does not hold. Every ```console block whose single command
is `typikon ...` is executed and compared line for line.

    tests/readme-examples.py            # against ./target/release/typikon
    TYPIKON=/usr/bin/typikon tests/readme-examples.py

A block ending in a horizontal ellipsis is treated as a prefix, for listings
too long to paste whole.
"""
import difflib
import os
import re
import shlex
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BINARY = os.environ.get("TYPIKON") or str(ROOT / "target" / "release" / "typikon")


def commands_and_body(block):
    """Split a console block into its commands and its expected output,
    joining backslash continuations."""
    out, buf = [], ""
    for line in block.rstrip("\n").split("\n"):
        if buf:
            buf += " " + line.strip().rstrip("\\ ")
            if not line.rstrip().endswith("\\"):
                out.append("$ " + buf)
                buf = ""
            continue
        if line.startswith("$ ") and line.rstrip().endswith("\\"):
            buf = line[2:].rstrip("\\ ")
        else:
            out.append(line)
    if buf:
        out.append("$ " + buf)
    cmds = [l[2:] for l in out if l.startswith("$ ")]
    body = "\n".join(l for l in out if not l.startswith("$ ")).strip("\n")
    return cmds, body


def main():
    if not Path(BINARY).exists():
        print(f"  {BINARY} not built; run cargo build --release", file=sys.stderr)
        return 77
    env = dict(os.environ, COLUMNS="88")
    # The examples must not depend on whatever this machine remembers, nor on
    # a menaion this machine happens to have fetched. A reader running them
    # after a fresh install has neither, and the README should show what they
    # get: without this the Ruthenian examples passed here and failed in CI.
    env.pop("TYPIKON_JURISDICTION", None)
    env["XDG_CONFIG_HOME"] = str(ROOT / "target" / "readme-config")
    env["XDG_DATA_HOME"] = str(ROOT / "target" / "readme-data")

    text = (ROOT / "README.md").read_text(encoding="utf-8")
    checked = bad = skipped = 0
    for block in re.findall(r"```console\n(.*?)```", text, re.S):
        cmds, expected = commands_and_body(block)
        if len(cmds) != 1 or not cmds[0].startswith("typikon") or not expected:
            skipped += 1
            continue
        argv = [BINARY] + shlex.split(cmds[0])[1:]
        got = subprocess.run(argv, capture_output=True, text=True, env=env).stdout.strip("\n")
        checked += 1
        if expected.rstrip().endswith("…"):
            ok = got.startswith(expected.rstrip()[:-1].rstrip("\n"))
        else:
            ok = got == expected
        if not ok:
            bad += 1
            print(f"  MISMATCH  {cmds[0]}")
            for line in difflib.unified_diff(
                expected.split("\n"), got.split("\n"), "README", "actual", lineterm=""
            ):
                print("   ", line)
    print(f"\n  {checked} README examples run, {bad} mismatched, {skipped} not executable")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
