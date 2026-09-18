// A bare "typikon" import under a browser-targeted bundler, which is what a
// web app actually writes. It has to resolve to the web build and not the Node
// one: the exports map is the only thing making that true.
import init, { describeIso } from "typikon";
import wasmUrl from "typikon/web/typikon_bg.wasm";
import { shape } from "../shape.mjs";

window.typikonProbe = async (probes) => {
  await init({ module_or_path: wasmUrl });
  const out = {};
  for (const [date, jurisdiction, typikon] of probes) {
    out[`${date}|${jurisdiction}|${typikon ? 1 : 0}`] =
      shape(describeIso(date, { jurisdiction, typikon }));
  }
  return out;
};
