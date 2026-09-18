"""One surface's answers against the oracle's.

A surface need not expose every field; it must not disagree about one it does.
"""
import json
import sys

name, oracle_path, got_path = sys.argv[1:4]
oracle = json.load(open(oracle_path))
got = json.load(open(got_path))

bad = []
for key in sorted(set(oracle) - set(got)):
    bad.append(f"missing probe {key}")
for key in sorted(set(got) - set(oracle)):
    bad.append(f"unexpected probe {key}")
for key in sorted(set(oracle) & set(got)):
    for field, expected in sorted(oracle[key].items()):
        if field in got[key] and got[key][field] != expected:
            bad.append(f"{key} {field}: {got[key][field]!r} != {expected!r}")

fields = sorted({f for v in got.values() for f in v})
for line in bad[:20]:
    print(f"      {line}")
if len(bad) > 20:
    print(f"      ... and {len(bad) - 20} more")
print(f"  {'ok  ' if not bad else 'FAIL'} {name}: "
      f"{len(got)} probes, {len(fields)} fields, {len(bad)} differences")
sys.exit(1 if bad else 0)
