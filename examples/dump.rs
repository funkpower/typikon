// Dump every day of a range, every jurisdiction, as one JSON object per line,
// for differential testing against the Python implementation.
use typikon::calendars::Date;
use typikon::day::describe_with;
use typikon::jurisdictions::{CALENDARS, JURISDICTIONS};
use typikon::menologion;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let start = Date::parse_iso(&args[1]).expect("start date");
    let days: i32 = args[2].parse().expect("days");
    for jur in JURISDICTIONS.iter().chain(CALENDARS.iter()) {
        let men = menologion::for_jurisdiction(jur.menaia);
        for n in 0..days {
            let d = describe_with(start + n, jur, &men);
            println!("{}", serde_json::to_string(&d.to_json()).unwrap());
        }
    }
}
