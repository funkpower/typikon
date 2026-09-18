#!/usr/bin/env bash
# Every binding, asked the same twelve days, compared field by field against
# what the command line says.
#
#     tests/parity.sh              # against what is in this checkout
#     tests/parity.sh --published  # against what is on PyPI and npm
#
# A surface whose toolchain is missing is skipped by name. A surface that runs
# and disagrees fails the script. tests/parity/README.md says why these twelve.
set -uo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
P=tests/parity
published=0
[ "${1:-}" = "--published" ] && published=1

work="$(mktemp -d)"
server=""
# By pid. A job spec does not survive a subshell, and a web server left holding
# the script's stdout is a suite that hangs rather than fails.
cleanup() {
  [ -n "$server" ] && kill "$server" 2>/dev/null && wait "$server" 2>/dev/null
  rm -rf "$work"
}
trap cleanup EXIT

pass=0 fail=0 skipped=()
note()   { printf '\n\033[1m%s\033[0m\n' "$1"; }
skip()   { skipped+=("$1: $2"); printf '  skip %s (%s)\n' "$1" "$2"; }
verify() { # name, path to that surface's json
  if python3 "$P/compare.py" "$1" "$work/oracle.json" "$2"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
  fi
}

# Neither a config file nor a fetched menaion may reach any surface, or this
# measures the machine rather than the code.
#
# Empty directories, not missing ones. Pointing these at a path that does not
# exist isolates typikon and breaks everything else that keeps state under
# them: a browser cannot build a profile and its crash handler will not start
# without somewhere to write.
export XDG_CONFIG_HOME="$work/xdg/config" XDG_DATA_HOME="$work/xdg/data"
mkdir -p "$XDG_CONFIG_HOME" "$XDG_DATA_HOME"

# ── the oracle ────────────────────────────────────────────────────────────
note "oracle"
cargo build --quiet --release || exit 1
python3 "$P/oracle.py" ./target/release/typikon "$P/probes.json" "$work/oracle.json" || exit 1

# The C probe carries its own copy of the list, because a binding whose point
# is to spare its callers a JSON parser should not need one to be tested.
python3 - "$P/probes.json" "$P/probe.c" <<'PY' || exit 1
import json, re, sys
probes = json.load(open(sys.argv[1]))
src = open(sys.argv[2]).read()
found = re.findall(r"\{(\d+),(\d+),(\d+),\"([a-z-]+)\",(-?\d+)\}", src.replace(" ", ""))
mine = [(f"{int(y):04d}-{int(m):02d}-{int(d):02d}", j, t == "1") for y, m, d, j, t in found]
want = [(d, j, bool(t)) for d, j, t in probes]
if mine != want:
    sys.exit(f"tests/parity/probe.c has drifted from probes.json:\n  c    {mine}\n  json {want}")
PY

# ── python ────────────────────────────────────────────────────────────────
note "python"
py="$work/venv/bin/python"
if ! python3 -m venv "$work/venv" >"$work/py.log" 2>&1; then
  skip python "no usable python3 -m venv"
elif [ "$published" = 1 ] && ! "$work/venv/bin/pip" install --quiet typikon >>"$work/py.log" 2>&1; then
  skip python "typikon would not install from PyPI"
elif [ "$published" = 0 ] && ! {
       "$work/venv/bin/pip" install --quiet maturin >>"$work/py.log" 2>&1 &&
       "$work/venv/bin/maturin" build --release --out "$work/wheels" >>"$work/py.log" 2>&1 &&
       "$work/venv/bin/pip" install --quiet --no-index --find-links "$work/wheels" typikon \
         >>"$work/py.log" 2>&1
     }; then
  echo "  FAIL python: the wheel would not build or install"
  tail -12 "$work/py.log" | sed 's/^/      /'
  fail=$((fail + 1))
else
  "$py" "$P/probe.py" "$P/probes.json" "$work/python.json" \
    && verify "python" "$work/python.json"
  # The stubs are part of what is published, so they are part of what is
  # checked. --strict, because a stub that only passes loosely is not one.
  if "$work/venv/bin/pip" install --quiet mypy >>"$work/py.log" 2>&1; then
    if "$py" -m mypy --strict "$P/probe.py" >"$work/mypy.log" 2>&1; then
      echo "  ok   python: mypy --strict"
    else
      echo "  FAIL python: mypy --strict"
      tail -8 "$work/mypy.log" | sed 's/^/      /'
      fail=$((fail + 1))
    fi
  else
    skip "python, mypy" "mypy would not install"
  fi
fi

# ── javascript ────────────────────────────────────────────────────────────
note "javascript"
if ! command -v node >/dev/null; then
  skip javascript "no node"
else
  js="$work/js"
  mkdir -p "$js"
  printf '{"name":"parity","private":true,"type":"module"}\n' > "$js/package.json"
  cp "$P/probes.json" "$P/shape.mjs" "$P"/probe.{mjs,cjs,ts} "$js/"
  mkdir -p "$js/web" && cp "$P"/web/{entry.mjs,index.html,extract.py} "$js/web/"

  if [ "$published" = 1 ]; then
    (cd "$js" && npm install --silent typikon >/dev/null 2>&1)
  else
    npm/build.sh >/dev/null 2>&1 && (cd "$js" && npm install --silent "$root/npm/dist" >/dev/null 2>&1)
  fi

  if [ ! -d "$js/node_modules/typikon" ]; then
    skip javascript "the package would not build or install"
  else
    node "$js/probe.mjs" "$js/probes.json" "$work/node-esm.json" \
      && verify "node, import" "$work/node-esm.json"
    node "$js/probe.cjs" "$js/probes.json" "$work/node-cjs.json" \
      && verify "node, require" "$work/node-cjs.json"

    # TypeScript: compiled under --strict, then run.
    if (cd "$js" && npm install --silent --no-save typescript@5 @types/node >/dev/null 2>&1); then
      if (cd "$js" && ./node_modules/.bin/tsc --strict --target es2022 --module nodenext \
            --moduleResolution nodenext --types node --outDir out probe.ts >/dev/null); then
        node "$js/out/probe.js" "$js/probes.json" "$work/typescript.json" \
          && verify "typescript, tsc --strict then run" "$work/typescript.json"
      else
        echo "  FAIL typescript: tsc --strict rejected the published types"; fail=$((fail + 1))
      fi
    else
      skip typescript "typescript would not install"
    fi

    # The browser, which is the only place the web build is really exercised:
    # a real fetch of the wasm, and no Node to fall back on.
    chrome=""
    for c in chromium chromium-browser google-chrome google-chrome-stable; do
      command -v "$c" >/dev/null && { chrome="$c"; break; }
    done
    if [ -z "$chrome" ]; then
      skip browser "no chromium"
    elif ! (cd "$js" && npm install --silent --no-save esbuild >/dev/null 2>&1); then
      skip browser "esbuild would not install"
    else
      if ! (cd "$js" && ./node_modules/.bin/esbuild web/entry.mjs --bundle \
              --format=esm --platform=browser --loader:.wasm=file \
              --outdir=served --log-level=error); then
        echo "  FAIL browser: the bundle would not build"; fail=$((fail + 1))
      else
        cp "$js/web/index.html" "$js/probes.json" "$js/served/"
        # A port the kernel picks, so a leftover server from a previous run
        # cannot quietly serve a stale bundle to this one.
        port=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1]); s.close()')
        python3 -m http.server "$port" --bind 127.0.0.1 \
          --directory "$js/served" >/dev/null 2>&1 &
        server=$!
        for _ in $(seq 40); do
          curl -sf -o /dev/null "http://127.0.0.1:$port/index.html" && break
          sleep 0.25
        done
        # --user-data-dir because this script points XDG_CONFIG_HOME at
        # nothing, to keep a real config file away from the surfaces under
        # test, and a browser given nowhere to put a profile does not start.
        # --disable-dev-shm-usage because a container's /dev/shm is too small.
        # Bounded: a browser that never paints must fail the suite, not hang it.
        timeout 90 "$chrome" --headless --disable-gpu --no-sandbox \
          --no-first-run --disable-dev-shm-usage \
          --user-data-dir="$work/chrome" \
          --virtual-time-budget=20000 --dump-dom "http://127.0.0.1:$port/index.html" \
          >"$work/dom.html" 2>"$work/chrome.log"
        kill "$server" 2>/dev/null; wait "$server" 2>/dev/null; server=""
        if python3 "$P/web/extract.py" "$work/dom.html" "$work/browser.json"; then
          verify "browser, bare import of the web build" "$work/browser.json"
        else
          echo "  FAIL browser"
          tail -5 "$work/chrome.log" 2>/dev/null | sed 's/^/      /'
          fail=$((fail + 1))
        fi
      fi
    fi

    for rt in deno bun; do
      command -v "$rt" >/dev/null || { skip "$rt" "not installed"; continue; }
      case "$rt" in
        deno) (cd "$js" && deno run --quiet --allow-read --allow-write --allow-env \
                 probe.mjs probes.json "$work/deno.json") ;;
        bun)  (cd "$js" && bun run probe.mjs probes.json "$work/bun.json") ;;
      esac && verify "$rt" "$work/$rt.json"
    done
  fi
fi

# ── the C ABI, and Lua over it ────────────────────────────────────────────
note "C and Lua"
if ! command -v cc >/dev/null; then
  skip "C ABI" "no C compiler"
elif ! cargo build --quiet --release --features capi; then
  echo "  FAIL C ABI: the library would not build"; fail=$((fail + 1))
else
  if cc -Wall -Wextra -Werror -Iinclude "$P/probe.c" \
       -Ltarget/release -ltypikon -o "$work/c-probe" 2>"$work/cc.log"; then
    LD_LIBRARY_PATH=target/release "$work/c-probe" "$work/c.json" \
      && verify "C ABI" "$work/c.json"
  else
    echo "  FAIL C ABI: the probe would not compile"; sed 's/^/      /' "$work/cc.log"
    fail=$((fail + 1))
  fi
  # The worked example has to keep working too.
  if cc -Wall -Wextra -Werror -Iinclude examples/c/demo.c \
       -Ltarget/release -ltypikon -o "$work/demo" 2>/dev/null \
     && LD_LIBRARY_PATH=target/release "$work/demo" rocor >/dev/null; then
    echo "  ok   C ABI: examples/c/demo.c"
  else
    echo "  FAIL C ABI: examples/c/demo.c"; fail=$((fail + 1))
  fi

  if ! command -v luajit >/dev/null; then
    skip luajit "not installed"
  else
    TYPIKON_LIB=target/release/libtypikon.so TYPIKON_LUA=examples/lua \
      luajit "$P/probe.lua" "$work/lua.json" && verify "luajit" "$work/lua.json"
    TYPIKON_LIB=target/release/libtypikon.so \
      luajit examples/lua/statusline.lua rocor >/dev/null \
      && echo "  ok   luajit: examples/lua/statusline.lua" \
      || { echo "  FAIL luajit: examples/lua/statusline.lua"; fail=$((fail + 1)); }
  fi
fi

# ── verdict ───────────────────────────────────────────────────────────────
note "verdict"
printf '  %d surfaces agree with the command line, %d disagree\n' "$pass" "$fail"
for s in "${skipped[@]:-}"; do [ -n "$s" ] && printf '  not run: %s\n' "$s"; done

# A run that skipped everything must not read as a pass.
if [ "$pass" -lt 3 ]; then
  echo "  too few surfaces ran to call this a check"
  exit 1
fi
exit $(( fail > 0 ))
