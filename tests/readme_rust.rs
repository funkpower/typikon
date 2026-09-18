//! The README's Rust example, compiled and asserted, so it cannot quietly
//! stop being true. The prose version elides the error handling; this is the
//! same calls with the results checked.

use typikon::calendars::Date;
use typikon::day::describe;
use typikon::fasting::Level;
use typikon::jurisdictions::resolve;

#[test]
fn the_readme_library_example_is_true() {
    let rocor = resolve(Some("rocor")).unwrap();
    let day = describe(Date::from_ymd(2026, 9, 18).unwrap(), rocor);

    assert_eq!(day.church_date, "5 September");
    assert_eq!(day.fast, Level::Xerophagy);
    assert_eq!(day.title(), "The Holy Prophet Zachariah, father of the Forerunner");
    assert!(!day.is_obligatory());

    assert_eq!(typikon::paschalion::julian_pascha(2027).iso(), "2027-05-02");
    assert_eq!(typikon::paschalion::gregorian_pascha(2027).iso(), "2027-03-28");
}

#[test]
fn the_readme_level_example_is_true() {
    // No saint brings fish into Lent.
    assert_eq!(Level::Xerophagy.relaxed_to(Level::Fish, Level::WineOil), Level::WineOil);
    // And relaxing never tightens.
    assert_eq!(Level::Fish.relaxed_to(Level::WineOil, Level::WineOil), Level::Fish);
}
