//! JSON, which is how every front end reads this.
//!
//! The shape is flat and stable: front ends are the thing most likely to be
//! written by someone else, and a schema that shifts underneath them is a bad
//! neighbour. One day is an object; several are `{"days": [...]}`.

use serde::Serialize;

use crate::day::{Day, DayJson};

#[derive(Serialize)]
struct Days<'a> {
    days: Vec<DayJson<'a>>,
}

pub fn to_json(days: &[Day], indent: Option<usize>) -> String {
    match days {
        [one] => encode(&one.to_json(), indent),
        many => encode(
            &Days { days: many.iter().map(Day::to_json).collect() },
            indent,
        ),
    }
}

fn encode<T: Serialize>(value: &T, indent: Option<usize>) -> String {
    match indent {
        None => serde_json::to_string(value).expect("a day serialises"),
        Some(n) => {
            let pad = vec![b' '; n];
            let fmt = serde_json::ser::PrettyFormatter::with_indent(&pad);
            let mut buf = Vec::new();
            let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
            value.serialize(&mut ser).expect("a day serialises");
            String::from_utf8(buf).expect("serde emits utf-8")
        }
    }
}
