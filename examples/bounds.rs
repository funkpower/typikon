fn main() {
    use typikon::calendars::{Date, MAX_YEAR, MIN_YEAR};
    println!("MIN {:?} = {:?}", Date::MIN, Date::MIN.ymd());
    println!("MAX {:?} = {:?}", Date::MAX, Date::MAX.ymd());
    println!("from_ymd(1,1,1)     {:?}", Date::from_ymd(MIN_YEAR, 1, 1).map(|d| d.iso()));
    println!("from_ymd(9999,12,31){:?}", Date::from_ymd(MAX_YEAR, 12, 31).map(|d| d.iso()));
    println!("from_ymd(10000,1,1) {:?}", Date::from_ymd(10000, 1, 1).map(|d| d.iso()));
    println!("from_ymd(0,1,1)     {:?}", Date::from_ymd(0, 1, 1).map(|d| d.iso()));
}
