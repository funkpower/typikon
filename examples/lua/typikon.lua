-- typikon, through LuaJIT's FFI.
--
-- Drop this beside your config and `local typikon = require("typikon")`. It
-- needs libtypikon.so on the loader's path, or TYPIKON_LIB set to it:
--
--     cargo build --release --features capi
--     TYPIKON_LIB=target/release/libtypikon.so luajit examples/lua/statusline.lua
--
-- Written for AwesomeWM, WezTerm and Neovim, which all embed LuaJIT and none
-- of which should be spawning a process on every redraw.

local ffi = require("ffi")

ffi.cdef [[
const char *typikon_version(void);
uint32_t typikon_abi_version(void);
const char *typikon_last_error(void);

int32_t typikon_jdn_from_ymd(int32_t year, uint32_t month, uint32_t day);
int typikon_ymd_from_jdn(int32_t jdn, int32_t *year, uint32_t *month, uint32_t *day);
int32_t typikon_today(void);

typedef struct typikon_day typikon_day;
typikon_day *typikon_describe(int32_t jdn, const char *jurisdiction);
typikon_day *typikon_describe_full(int32_t jdn, const char *jurisdiction,
                                   const char *tradition, int discipline);
void typikon_day_free(typikon_day *day);

const char *typikon_day_json(const typikon_day *day);
const char *typikon_day_iso(const typikon_day *day);
const char *typikon_day_weekday(const typikon_day *day);
const char *typikon_day_jurisdiction(const typikon_day *day);
const char *typikon_day_calendar(const typikon_day *day);
const char *typikon_day_tradition(const typikon_day *day);
const char *typikon_day_church_date(const typikon_day *day);
const char *typikon_day_title(const typikon_day *day);
const char *typikon_day_fast(const typikon_day *day);
const char *typikon_day_fast_label(const typikon_day *day);
const char *typikon_day_emoji(const typikon_day *day);
const char *typikon_day_reason(const typikon_day *day);
const char *typikon_day_season(const typikon_day *day);
const char *typikon_day_cycle(const typikon_day *day);
const char *typikon_day_week(const typikon_day *day);
const char *typikon_day_feast(const typikon_day *day);
const char *typikon_day_rank(const typikon_day *day);
const char *typikon_day_sign(const typikon_day *day);
const char *typikon_day_obligation(const typikon_day *day);
const char *typikon_day_binds(const typikon_day *day);
uint32_t typikon_day_commemoration_count(const typikon_day *day);
const char *typikon_day_commemoration(const typikon_day *day, uint32_t i);
int typikon_day_is_obligatory(const typikon_day *day);
int32_t typikon_day_days_to_pascha(const typikon_day *day);
int32_t typikon_day_pascha(const typikon_day *day);
int32_t typikon_day_julian_offset(const typikon_day *day);

int32_t typikon_pascha(int32_t year, const char *jurisdiction);
uint32_t typikon_jurisdiction_count(void);
const char *typikon_jurisdiction_key(uint32_t i);
const char *typikon_jurisdiction_name(uint32_t i);
const char *typikon_jurisdiction_note(uint32_t i);
]]

local C = ffi.load(os.getenv("TYPIKON_LIB") or "typikon")

local M = { ABI = 1 }

-- A string field, or nil where the library returned NULL. Every accessor may,
-- and NULL means "the day has no such thing" rather than an error.
local function str(p)
  if p == nil then return nil end
  return ffi.string(p)
end

local function err()
  return str(C.typikon_last_error()) or "unknown error"
end

-- The fields worth reading without a JSON parser. Order matters only for the
-- table's own readability.
local FIELDS = {
  "iso", "weekday", "jurisdiction", "calendar", "tradition", "church_date",
  "title", "fast", "fast_label", "emoji", "reason",
  "season", "cycle", "week", "feast", "rank", "sign", "obligation", "binds",
}

-- Copy the day into a plain Lua table and free it. The C day owns its strings
-- and they die with it, so nothing here may hold on to a pointer.
local function harvest(d)
  local t = {}
  for _, f in ipairs(FIELDS) do
    t[f] = str(C["typikon_day_" .. f](d))
  end
  t.is_obligatory = C.typikon_day_is_obligatory(d) ~= 0
  t.days_to_pascha = C.typikon_day_days_to_pascha(d)
  t.pascha_jdn = C.typikon_day_pascha(d)
  t.julian_offset = C.typikon_day_julian_offset(d)
  t.commemorations = {}
  for i = 0, C.typikon_day_commemoration_count(d) - 1 do
    t.commemorations[i + 1] = str(C.typikon_day_commemoration(d, i))
  end
  t.json = str(C.typikon_day_json(d))
  C.typikon_day_free(d)
  return t
end

--- One day, as a table. `opts` may carry `jurisdiction`, `tradition` and
--- `typikon` (true to show the day at the Typikon's full discipline, false for
--- only what particular law requires, nil for whatever was remembered).
--- Returns nil and a message on error.
function M.describe(jdn, opts)
  opts = opts or {}
  local discipline = -1
  if opts.typikon == true then discipline = 1
  elseif opts.typikon == false then discipline = 0 end
  local d = C.typikon_describe_full(jdn, opts.jurisdiction, opts.tradition, discipline)
  if d == nil then return nil, err() end
  return harvest(d)
end

--- Today, by local time.
function M.today(opts)
  return M.describe(C.typikon_today(), opts)
end

--- The Julian Day Number for a civil date, or nil and a message.
function M.jdn(year, month, day)
  local n = C.typikon_jdn_from_ymd(year, month, day)
  if n == 0 then return nil, err() end
  return n
end

--- The civil year, month and day for a Julian Day Number.
function M.ymd(jdn)
  local y = ffi.new("int32_t[1]")
  local m = ffi.new("uint32_t[1]")
  local d = ffi.new("uint32_t[1]")
  if C.typikon_ymd_from_jdn(jdn, y, m, d) ~= 0 then return nil, err() end
  return y[0], tonumber(m[0]), tonumber(d[0])
end

--- Pascha in a civil year, as a Julian Day Number.
function M.pascha(year, jurisdiction)
  local n = C.typikon_pascha(year, jurisdiction)
  if n == 0 then return nil, err() end
  return n
end

--- Every jurisdiction the library knows: { key, name, note }.
function M.jurisdictions()
  local out = {}
  for i = 0, C.typikon_jurisdiction_count() - 1 do
    out[i + 1] = {
      key = str(C.typikon_jurisdiction_key(i)),
      name = str(C.typikon_jurisdiction_name(i)),
      note = str(C.typikon_jurisdiction_note(i)),
    }
  end
  return out
end

M.version = str(C.typikon_version())

-- A library newer in a way that changed meaning, not one merely newer.
if C.typikon_abi_version() ~= M.ABI then
  error(("typikon ABI %d, this binding wants %d")
    :format(tonumber(C.typikon_abi_version()), M.ABI))
end

return M
