"""The command line's answer, which every other surface is measured against."""
import json
import os
import subprocess
import sys

FIELDS = [
    "date", "weekday", "jurisdiction", "calendar", "churchDate", "season",
    "cycle", "week", "feast", "rank", "sign", "tradition", "title", "commemorations",
    "fast", "fastLabel", "emoji", "reason", "obligation", "binds", "pascha",
    "daysToPascha", "julianOffset",
]

cli, probes_path, out_path = sys.argv[1:4]

# Neither a config file nor a fetched menaion may reach any surface, or the
# comparison measures the machine rather than the code. The caller has already
# pointed these at empty directories; this only refuses to run without them,
# because an oracle that quietly read the developer's own jurisdiction would
# make every surface agree about the wrong thing.
env = dict(os.environ)
for var in ("XDG_CONFIG_HOME", "XDG_DATA_HOME"):
    if not env.get(var):
        sys.exit(f"{var} must be set to an empty directory before reckoning the oracle")

out = {}
for date, jurisdiction, typikon in json.load(open(probes_path)):
    args = [cli, "--date", date, "--jurisdiction", jurisdiction, "--json",
            "--typikon" if typikon else "--no-typikon"]
    day = json.loads(subprocess.run(args, capture_output=True, text=True,
                                    env=env, check=True).stdout)
    out[f"{date}|{jurisdiction}|{int(typikon)}"] = {k: day[k] for k in FIELDS}

json.dump(out, open(out_path, "w"), ensure_ascii=False)
print(f"oracle: {len(out)} probes, {len(FIELDS)} fields")
