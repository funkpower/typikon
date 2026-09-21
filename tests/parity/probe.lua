-- The binding under test is the one in examples/lua.
package.path = (os.getenv("TYPIKON_LUA") or "examples/lua") .. "/?.lua;" .. package.path
local typikon = require("typikon")

-- Minimal JSON writer: the binding hands back Lua tables, and the point is to
-- compare them with everything else, not to pull in a dependency.
local function esc(s)
  return '"' .. s:gsub('[%c"\\]', function(c)
    if c == '"' then return '\\"' elseif c == '\\' then return '\\\\' end
    return string.format('\\u%04x', c:byte())
  end) .. '"'
end
local function val(v)
  if v == nil then return "null"
  elseif type(v) == "number" then return tostring(math.floor(v))
  elseif type(v) == "table" then
    local parts = {}
    for i = 1, #v do parts[i] = esc(v[i]) end
    return "[" .. table.concat(parts, ",") .. "]"
  else return esc(v) end
end

local probes = {
  {2026,4,12,"moscow",false}, {2026,4,12,"goarch",false}, {2026,1,7,"moscow",false},
  {2025,12,25,"old",false},   {2026,3,4,"ruthenian",false},{2026,3,4,"ruthenian",true},
  {2025,12,9,"old",false},    {2026,2,16,"oca",false},     {2026,8,15,"antiochian",false},
  {2026,9,18,"finnish",false},{2027,5,2,"serbian",false},  {2026,6,28,"constantinople",false},
}

local fields = {
  {"date","iso"}, {"weekday","weekday"}, {"jurisdiction","jurisdiction"},
  {"calendar","calendar"}, {"churchDate","church_date"}, {"season","season"},
  {"cycle","cycle"}, {"week","week"}, {"feast","feast"}, {"rank","rank"}, {"sign","sign"},
  {"tradition","tradition"}, {"title","title"}, {"fast","fast"},
  {"fastLabel","fast_label"}, {"emoji","emoji"}, {"reason","reason"},
  {"obligation","obligation"}, {"binds","binds"},
  {"commemorations","commemorations"},
}

local out, firstkey = {}, true
for _, p in ipairs(probes) do
  local jdn = assert(typikon.jdn(p[1], p[2], p[3]))
  local day = assert(typikon.describe(jdn, { jurisdiction = p[4], typikon = p[5] }))
  local parts = {}
  for _, f in ipairs(fields) do
    parts[#parts + 1] = esc(f[1]) .. ":" .. val(day[f[2]])
  end
  local py, pm, pd = typikon.ymd(day.pascha_jdn)
  parts[#parts + 1] = esc("pascha") .. ":" .. esc(string.format("%04d-%02d-%02d", py, pm, pd))
  parts[#parts + 1] = esc("daysToPascha") .. ":" .. tostring(day.days_to_pascha)
  parts[#parts + 1] = esc("julianOffset") .. ":" .. tostring(day.julian_offset)
  local key = string.format("%04d-%02d-%02d|%s|%d", p[1], p[2], p[3], p[4], p[5] and 1 or 0)
  out[#out + 1] = (firstkey and "" or ",") .. esc(key) .. ":{" .. table.concat(parts, ",") .. "}"
  firstkey = false
end

local f = assert(io.open(arg[1], "w"))
f:write("{" .. table.concat(out) .. "}\n")
f:close()
