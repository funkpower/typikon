/* typikon - the Orthodox liturgical day, computed rather than fetched.
 *
 * SPDX-License-Identifier: GPL-2.0-only
 * https://git.pumpin.online/funkpower/typikon
 *
 * Link against libtypikon.so, built with:
 *
 *     cargo build --release --features capi
 *
 * Dates cross this boundary as Julian Day Numbers, so no struct layout is part
 * of the ABI. typikon_jdn_from_ymd converts.
 *
 * A typikon_day is opaque and owns its strings. They are built once when the
 * day is reckoned, so an accessor returns the same borrowed, NUL-terminated
 * pointer every time and allocates nothing: calling one in a render loop is
 * free. A pointer is valid until typikon_day_free, or NULL where the day has
 * no such field. Nothing an accessor returns should be freed.
 *
 * A call that can fail returns NULL or 0 and leaves a message in
 * typikon_last_error(), which is thread-local.
 *
 * Every function is safe to call from any thread. A typikon_day is not
 * synchronised; do not share one between threads without your own lock.
 */

#ifndef TYPIKON_H
#define TYPIKON_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ---- version ---------------------------------------------------------- */

/* A dotted version string. Static storage; never freed. */
const char *typikon_version(void);

/* Bumped when anything here changes meaning or signature. Additions do not
 * bump it. Compare against TYPIKON_ABI_VERSION if you load at runtime. */
uint32_t typikon_abi_version(void);
#define TYPIKON_ABI_VERSION 1u

/* The message from the last failing call on this thread, or NULL if the last
 * call succeeded. Valid until the next failing call on this thread. */
const char *typikon_last_error(void);

/* ---- dates ------------------------------------------------------------ */

/* Julian Day Number for a civil (Gregorian) date, or 0 if there is no such
 * date or it falls outside the years 1 to 9999. */
int32_t typikon_jdn_from_ymd(int32_t year, uint32_t month, uint32_t day);

/* Civil date for a Julian Day Number. Returns 0 on success, -1 if the number
 * is outside the supported years. Any out-param may be NULL. */
int typikon_ymd_from_jdn(int32_t jdn, int32_t *year, uint32_t *month,
                         uint32_t *day);

/* Today's Julian Day Number, in local time. */
int32_t typikon_today(void);

/* ---- a day ------------------------------------------------------------ */

typedef struct typikon_day typikon_day;

/* Reckon a day. jurisdiction may be NULL to use whatever
 * `typikon --set-default` remembered. NULL on error. Free with
 * typikon_day_free. */
typikon_day *typikon_describe(int32_t jdn, const char *jurisdiction);

/* As typikon_describe, overriding the usage and the discipline.
 *
 *   tradition   "slavic", "greek", or NULL for the jurisdiction's own.
 *   discipline   1  show the day at the Typikon's full discipline
 *                0  show only what the church's particular law requires
 *               -1  use whatever was remembered
 *
 * The distinction matters for the Byzantine Catholic churches, whose law
 * requires very much less than the tradition they keep. */
typikon_day *typikon_describe_full(int32_t jdn, const char *jurisdiction,
                                   const char *tradition, int discipline);

/* NULL is accepted and ignored. */
void typikon_day_free(typikon_day *day);

/* ---- reading a day ---------------------------------------------------- */

/* The whole day as JSON: the same document the command line emits, under the
 * same stability promise. Use this if you have a JSON parser to hand; the
 * accessors below carry every field of it, so that a statusline does not need
 * one. */
const char *typikon_day_json(const typikon_day *day);

/* Never NULL for a live day. */
const char *typikon_day_iso(const typikon_day *day);          /* "2026-04-12" */
const char *typikon_day_weekday(const typikon_day *day);      /* in English */
const char *typikon_day_jurisdiction(const typikon_day *day); /* key of the church whose reckoning this is */
const char *typikon_day_calendar(const typikon_day *day);     /* old|new|gregorian */
const char *typikon_day_tradition(const typikon_day *day);    /* slavic|greek */
const char *typikon_day_church_date(const typikon_day *day);  /* "25 December" */
const char *typikon_day_title(const typikon_day *day);        /* feast, else principal commemoration */
const char *typikon_day_fast(const typikon_day *day);         /* strict|xerophagy|wine_oil|fish|dairy|none */
const char *typikon_day_fast_label(const typikon_day *day);   /* as this tradition words it */
const char *typikon_day_emoji(const typikon_day *day);        /* glyph for the level */
const char *typikon_day_reason(const typikon_day *day);       /* why the day fasts as it does */

/* NULL where the day has no such thing, which is not an error. */
const char *typikon_day_season(const typikon_day *day);       /* NULL in ordinary time */
const char *typikon_day_cycle(const typikon_day *day);        /* NULL outside the movable cycle */
const char *typikon_day_week(const typikon_day *day);         /* NULL outside the count from Pentecost */
const char *typikon_day_feast(const typikon_day *day);        /* NULL if the day has none */
const char *typikon_day_rank(const typikon_day *day);         /* NULL if the day has no feast */
const char *typikon_day_sign(const typikon_day *day);         /* the mark the books print, else NULL */
const char *typikon_day_obligation(const typikon_day *day);   /* obligatory|customary, else NULL */
const char *typikon_day_binds(const typikon_day *day);        /* NULL where nothing binds */

/* The saints and events kept on the day, in the order the books give them. */
uint32_t typikon_day_commemoration_count(const typikon_day *day);
const char *typikon_day_commemoration(const typikon_day *day, uint32_t i);

/* Whether the day binds under the church's own particular law. */
int typikon_day_is_obligatory(const typikon_day *day);

/* Signed, to the Pascha of this day's own civil year. Negative once it has
 * passed: 31 December reports about -263, not the days until the next one. */
int32_t typikon_day_days_to_pascha(const typikon_day *day);

/* Julian Day Number of Pascha in this day's civil year. */
int32_t typikon_day_pascha(const typikon_day *day);

/* How far the Julian reckoning lags the Gregorian: 13 until 14 March 2100. */
int32_t typikon_day_julian_offset(const typikon_day *day);

/* ---- without a day ---------------------------------------------------- */

/* Julian Day Number of Pascha in a civil year, by a jurisdiction's
 * paschalion. jurisdiction may be NULL. 0 on error. */
int32_t typikon_pascha(int32_t year, const char *jurisdiction);

/* Every jurisdiction this build knows: the named churches first, then the
 * three bare calendars. The key is what typikon_describe takes. Static
 * storage; NULL past the end. */
uint32_t typikon_jurisdiction_count(void);
const char *typikon_jurisdiction_key(uint32_t i);
const char *typikon_jurisdiction_name(uint32_t i);
const char *typikon_jurisdiction_note(uint32_t i);

#ifdef __cplusplus
}
#endif

#endif /* TYPIKON_H */
