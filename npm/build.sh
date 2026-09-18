#!/usr/bin/env bash
# Build the npm package into npm/dist.
#
#     npm/build.sh            # needs wasm-bindgen-cli at the version below
#     WASM_BINDGEN=/path/to/wasm-bindgen npm/build.sh
#
# Two builds go in: `node/` for Node, which loads the module itself, and
# `web/` for browsers and bundlers, which cannot compile WebAssembly
# synchronously on the main thread and so has to be awaited. The exports map
# in package.json picks between them.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
bindgen="${WASM_BINDGEN:-wasm-bindgen}"
dist="npm/dist"

# The generator has to be the exact version the crate was built against, or it
# refuses the module. Cargo.lock is where that version actually is.
want="$(awk '/^name = "wasm-bindgen"$/{getline; gsub(/[",]/,""); print $3; exit}' Cargo.lock)"
command -v "$bindgen" >/dev/null || {
  echo "wasm-bindgen is not on PATH." >&2
  echo "  cargo install wasm-bindgen-cli --version $want" >&2
  echo "or take the prebuilt one from" >&2
  echo "  https://github.com/rustwasm/wasm-bindgen/releases/tag/$want" >&2
  exit 1
}
have="$("$bindgen" --version | awk '{print $2}')"
[ "$have" = "$want" ] || {
  echo "wasm-bindgen $have, but Cargo.lock pins $want" >&2
  exit 1
}

cargo build --lib --release --target wasm32-unknown-unknown --features wasm

rm -rf "$dist"
for target in nodejs web; do
  out="$dist/$([ "$target" = nodejs ] && echo node || echo web)"
  "$bindgen" --target "$target" --out-dir "$out" --out-name typikon \
    target/wasm32-unknown-unknown/release/typikon.wasm
  # wasm-bindgen writes an overridden parameter type without the optional
  # marker, so an options argument nobody wants to pass has to be passed as
  # `undefined`. The override is what gives the argument a real type at all;
  # this restores the `?` it costs.
  sed -i 's/options: Options/options?: Options/g' "$out/typikon.d.ts"
done

# wasm-opt, where it is to hand. Worth about a fifth of the file.
if command -v wasm-opt >/dev/null; then
  for w in "$dist"/*/typikon_bg.wasm; do
    wasm-opt -Oz -o "$w.opt" "$w" && mv "$w.opt" "$w"
  done
fi

sed "s/@VERSION@/$version/" npm/package.json.in > "$dist/package.json"
# The package is ESM; the Node build is what wasm-bindgen emits for Node, which
# is CommonJS. One file says so for that directory alone.
echo '{ "type": "commonjs" }' > "$dist/node/package.json"
cp npm/README.md "$dist/README.md"
cp LICENSE "$dist/LICENSE"

echo "built $dist, version $version"
du -h "$dist"/*/typikon_bg.wasm
