"""JSON, which is how every front end reads this.

The shape is flat and stable: front ends are the thing most likely to be
written by someone else, and a schema that shifts underneath them is a bad
neighbour. One day is an object; several are ``{"days": [...]}``.
"""

import json

__all__ = ["to_json"]


def to_json(days, indent=None):
    if not isinstance(days, (list, tuple)):
        return json.dumps(days.to_dict(), ensure_ascii=False, indent=indent)
    if len(days) == 1:
        return json.dumps(days[0].to_dict(), ensure_ascii=False, indent=indent)
    return json.dumps({"days": [d.to_dict() for d in days]},
                      ensure_ascii=False, indent=indent)
