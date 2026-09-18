/* Build:
 *     cargo build --release --features capi
 *     cc -Iinclude examples/c/demo.c -Ltarget/release -ltypikon -o /tmp/demo
 *     LD_LIBRARY_PATH=target/release /tmp/demo rocor
 *
 * With no arguments it uses the remembered default; with --list it prints
 * every jurisdiction it knows.
 */
#include <stdio.h>
#include <string.h>
#include <typikon.h>

static void show(int32_t jdn, const char *jurisdiction) {
    typikon_day *d = typikon_describe(jdn, jurisdiction);
    if (!d) {
        fprintf(stderr, "  %s\n", typikon_last_error());
        return;
    }
    const char *season = typikon_day_season(d);
    const char *sign = typikon_day_sign(d);
    printf("  %-12s %-14s %-4s %-28s %s%s%s\n",
           jurisdiction ? jurisdiction : "(default)",
           typikon_day_church_date(d),
           typikon_day_emoji(d),
           typikon_day_fast_label(d),
           sign ? sign : "",
           typikon_day_title(d),
           typikon_day_is_obligatory(d) ? "  [obligatory]" : "");
    if (season) printf("               season: %s\n", season);
    /* The title is the feast, or the first commemoration where there is no
       feast, so start past it rather than printing it twice. */
    uint32_t start = typikon_day_feast(d) ? 0 : 1;
    for (uint32_t i = start; i < typikon_day_commemoration_count(d); i++)
        printf("               also %s\n", typikon_day_commemoration(d, i));
    typikon_day_free(d);
}

static void list(void) {
    for (uint32_t i = 0; i < typikon_jurisdiction_count(); i++)
        printf("  %-18s %s\n", typikon_jurisdiction_key(i),
               typikon_jurisdiction_name(i));
}

int main(int argc, char **argv) {
    printf("typikon %s, ABI %u\n\n", typikon_version(), typikon_abi_version());

    if (argc == 2 && strcmp(argv[1], "--list") == 0) {
        list();
        return 0;
    }

    int32_t today = typikon_today();
    for (int i = 1; i < argc; i++) show(today, argv[i]);
    if (argc == 1) show(today, NULL);

    /* A day with something to say: the first day of the Great Fast, 2026,
       under Byzantine Catholic law and then at the Typikon's discipline. */
    int32_t clean_monday = typikon_jdn_from_ymd(2026, 2, 16);
    printf("\n  Clean Monday 2026:\n");
    typikon_day *law = typikon_describe_full(clean_monday, "ruthenian", NULL, 0);
    typikon_day *fast = typikon_describe_full(clean_monday, "ruthenian", NULL, 1);
    printf("    law      %-30s binds %s\n",
           typikon_day_fast_label(law),
           typikon_day_binds(law) ? typikon_day_binds(law) : "nothing");
    printf("    typikon  %-30s binds %s\n",
           typikon_day_fast_label(fast),
           typikon_day_binds(fast) ? typikon_day_binds(fast) : "nothing");
    typikon_day_free(law);
    typikon_day_free(fast);

    /* Pascha without reckoning a day for it. */
    int32_t p = typikon_pascha(2026, "old");
    int32_t y, dd;
    uint32_t m, d2;
    typikon_ymd_from_jdn(p, &y, &m, &d2);
    dd = p - typikon_today();
    printf("\n  Pascha 2026 (Julian paschalion): %04d-%02u-%02u, %d days from today\n",
           y, m, d2, dd);

    /* Errors do not crash. */
    if (!typikon_describe(today, "atlantis"))
        printf("\n  bad jurisdiction: %s\n", typikon_last_error());
    if (!typikon_jdn_from_ymd(2026, 2, 30))
        printf("  bad date        : %s\n", typikon_last_error());
    return 0;
}
