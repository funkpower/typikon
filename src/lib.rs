use crate::calendars::Date;

pub mod calendars;
pub mod config;
pub mod day;
pub mod export;
pub mod fasting;
pub mod jurisdictions;
pub mod menaia;
pub mod menologion;
pub mod paschalion;
pub mod rules;
pub mod seasons;

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "capi")]
pub mod capi;

/// Today, from the system clock, in local time.
///
/// The local offset comes from `date +%z`, because working it out from
/// /etc/localtime means parsing TZif and this program would rather shell out
/// than carry a time zone database. Where `date` is absent, as in a scratch
/// container, the answer is UTC and it says so on stderr rather than quietly
/// reporting the wrong day either side of midnight.
pub fn local_today() -> Date {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let local = secs + local_offset_secs();
    // Unix epoch is JDN 2440588. The clock cannot reach either bound.
    Date::from_jdn(2440588 + local.div_euclid(86400) as i32).unwrap_or(Date::MIN)
}

/// Seconds east of UTC, or None when it could not be determined.
fn local_offset_secs() -> i64 {
    let Ok(out) = std::process::Command::new("date").arg("+%z").output() else {
        eprintln!("typikon: `date` is not on PATH; reckoning today in UTC");
        return 0;
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let s = text.trim();
    // "+HHMM". Checked as bytes so a short or multi-byte reply cannot panic a
    // slice, which a str index here would.
    let b = s.as_bytes();
    if b.len() == 5 && (b[0] == b'+' || b[0] == b'-') && b[1..].iter().all(u8::is_ascii_digit) {
        let h = (s[1..3].parse::<i64>()).unwrap_or(0);
        let m = (s[3..5].parse::<i64>()).unwrap_or(0);
        let mag = h * 3600 + m * 60;
        return if b[0] == b'-' { -mag } else { mag };
    }
    eprintln!("typikon: could not read the local time offset; reckoning today in UTC");
    0
}

