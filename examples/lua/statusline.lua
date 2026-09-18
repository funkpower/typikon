-- What a statusline would do with it: one line, no subprocess, no JSON.
--
--     cargo build --release --features capi
--     TYPIKON_LIB=target/release/libtypikon.so \
--       luajit examples/lua/statusline.lua rocor

package.path = (arg[0]:match("(.*/)") or "./") .. "?.lua;" .. package.path

local typikon = require("typikon")

local day, e = typikon.today({ jurisdiction = arg[1] })
if not day then
  io.stderr:write(e, "\n")
  os.exit(1)
end

-- The one line.
print(("%s  %s  %s%s"):format(
  day.emoji,
  day.church_date,
  day.sign and (day.sign .. " ") or "",
  day.title))

-- And the detail a tooltip would carry.
print()
print(("  %-14s %s (%s, %s)"):format("reckoning", day.jurisdiction, day.calendar, day.tradition))
print(("  %-14s %s%s"):format("fast", day.fast_label,
  day.is_obligatory and "  [obligatory]" or ""))
if day.binds then
  print(("  %-14s %s"):format("law requires", day.binds))
end
print(("  %-14s %s"):format("because", day.reason))
if day.season then print(("  %-14s %s"):format("season", day.season)) end
if day.cycle then print(("  %-14s %s"):format("cycle", day.cycle)) end
for i = 2, #day.commemorations do
  print(("  %-14s %s"):format(i == 2 and "also" or "", day.commemorations[i]))
end

print(("  %-14s %d days (%s)"):format("to Pascha", day.days_to_pascha,
  ("%04d-%02d-%02d"):format(typikon.ymd(day.pascha_jdn))))
