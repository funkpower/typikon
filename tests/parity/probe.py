"""The Python bindings."""
import datetime
import json
import sys

import typikon

probes_path, out_path = sys.argv[1:3]
out = {}
for date, jurisdiction, tk in json.load(open(probes_path)):
    y, m, d = map(int, date.split("-"))
    day = typikon.describe(datetime.date(y, m, d), jurisdiction=jurisdiction, typikon=tk)
    out[f"{date}|{jurisdiction}|{int(tk)}"] = {
        "date": day.iso,
        "weekday": day.weekday,
        "jurisdiction": day.jurisdiction,
        "calendar": day.calendar,
        "churchDate": day.church_date,
        "season": day.season,
        "cycle": day.cycle,
        "week": day.week,
        "feast": day.feast,
        "rank": day.rank,
        "sign": day.sign,
        "tradition": day.tradition,
        "title": day.title,
        "commemorations": day.commemorations,
        "fast": day.fast,
        "fastLabel": day.fast_label,
        "emoji": day.emoji,
        "reason": day.reason,
        "obligation": day.obligation,
        "binds": day.binds,
        "pascha": day.pascha.isoformat(),
        "daysToPascha": day.days_to_pascha,
        "julianOffset": day.julian_offset,
    }
json.dump(out, open(out_path, "w"), ensure_ascii=False)
