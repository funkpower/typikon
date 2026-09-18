"""Pull the probe result back out of the DOM the browser printed."""
import html
import json
import re
import sys

dom = open(sys.argv[1], encoding="utf-8").read()
match = re.search(r'<pre id="out">(.*?)</pre>', dom, re.S)
if not match:
    sys.exit("the page produced no result element")
text = html.unescape(match.group(1))
if text == "running":
    sys.exit("the page never finished; the wasm probably failed to load")
if text.startswith("ERROR"):
    sys.exit(text[:600])
json.dump(json.loads(text), open(sys.argv[2], "w"), ensure_ascii=False)
