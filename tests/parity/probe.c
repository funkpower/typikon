/* The C ABI, over the same probes, emitting the same JSON.
 *
 * The probe list is written out here rather than read from probes.json,
 * because a JSON parser is exactly what this binding exists to spare its
 * callers and it would be odd to need one to test it. tests/parity.sh checks
 * the two lists have not drifted apart.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <typikon.h>

static void esc(FILE *f, const char *s) {
    fputc('"', f);
    for (; *s; s++) {
        if (*s == '"' || *s == '\\') fprintf(f, "\\%c", *s);
        else if ((unsigned char)*s < 0x20) fprintf(f, "\\u%04x", *s);
        else fputc(*s, f);
    }
    fputc('"', f);
}

static void field(FILE *f, const char *k, const char *v, int *first) {
    if (!*first) fputc(',', f);
    *first = 0;
    esc(f, k); fputc(':', f);
    if (v) esc(f, v); else fputs("null", f);
}

int main(int argc, char **argv) {
    (void)argc;
    FILE *f = fopen(argv[1], "w");
    fputc('{', f);
    int firstkey = 1;

    /* date, jurisdiction, discipline: the same twelve as probes.json. */
    struct { int y, m, d; const char *j; int t; } probes[] = {
        {2026,4,12,"moscow",-1}, {2026,4,12,"goarch",-1}, {2026,1,7,"moscow",-1},
        {2025,12,25,"old",-1},   {2026,3,4,"ruthenian",0}, {2026,3,4,"ruthenian",1},
        {2025,12,9,"old",-1},    {2026,2,16,"oca",-1},     {2026,8,15,"antiochian",-1},
        {2026,9,18,"finnish",-1},{2027,5,2,"serbian",-1},  {2026,6,28,"constantinople",-1},
    };

    for (size_t i = 0; i < sizeof probes / sizeof *probes; i++) {
        int32_t jdn = typikon_jdn_from_ymd(probes[i].y, probes[i].m, probes[i].d);
        int disc = probes[i].t < 0 ? 0 : probes[i].t;   /* the CLI oracle passed --no-typikon */
        typikon_day *day = typikon_describe_full(jdn, probes[i].j, NULL, disc);
        if (!day) { fprintf(stderr, "%s\n", typikon_last_error()); return 1; }

        char key[64];
        snprintf(key, sizeof key, "%04d-%02d-%02d|%s|%d",
                 probes[i].y, probes[i].m, probes[i].d, probes[i].j,
                 probes[i].t == 1 ? 1 : 0);
        if (!firstkey) fputc(',', f);
        firstkey = 0;
        esc(f, key); fputs(":{", f);
        int first = 1;
        field(f, "date",        typikon_day_iso(day), &first);
        field(f, "weekday",     typikon_day_weekday(day), &first);
        field(f, "jurisdiction",typikon_day_jurisdiction(day), &first);
        field(f, "calendar",    typikon_day_calendar(day), &first);
        field(f, "churchDate",  typikon_day_church_date(day), &first);
        field(f, "season",      typikon_day_season(day), &first);
        field(f, "cycle",       typikon_day_cycle(day), &first);
        field(f, "week",        typikon_day_week(day), &first);
        field(f, "feast",       typikon_day_feast(day), &first);
        field(f, "rank",        typikon_day_rank(day), &first);
        field(f, "sign",        typikon_day_sign(day), &first);
        field(f, "tradition",   typikon_day_tradition(day), &first);
        field(f, "title",       typikon_day_title(day), &first);
        field(f, "fast",        typikon_day_fast(day), &first);
        field(f, "fastLabel",   typikon_day_fast_label(day), &first);
        field(f, "emoji",       typikon_day_emoji(day), &first);
        field(f, "reason",      typikon_day_reason(day), &first);
        field(f, "obligation",  typikon_day_obligation(day), &first);
        field(f, "binds",       typikon_day_binds(day), &first);

        fputs(",\"commemorations\":[", f);
        for (uint32_t c = 0; c < typikon_day_commemoration_count(day); c++) {
            if (c) fputc(',', f);
            esc(f, typikon_day_commemoration(day, c));
        }
        fputc(']', f);

        int32_t p = typikon_day_pascha(day);
        int32_t py; uint32_t pm, pd;
        typikon_ymd_from_jdn(p, &py, &pm, &pd);
        char iso[16];
        snprintf(iso, sizeof iso, "%04d-%02u-%02u", py, pm, pd);
        fputs(",\"pascha\":", f); esc(f, iso);
        fprintf(f, ",\"daysToPascha\":%d", typikon_day_days_to_pascha(day));
        fprintf(f, ",\"julianOffset\":%d", typikon_day_julian_offset(day));
        fputc('}', f);
        typikon_day_free(day);
    }
    fputs("}\n", f);
    fclose(f);
    return 0;
}
