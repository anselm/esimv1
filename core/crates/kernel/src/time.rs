//! Virtual time in whole seconds since the Unix epoch, as i64. Field timesteps of
//! seconds to minutes fit; sub-second physics is not this kernel's business.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct Time(pub i64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct Duration(pub i64);

impl Time {
    pub const fn secs(s: i64) -> Self { Time(s) }
    pub fn add(self, d: Duration) -> Self { Time(self.0 + d.0) }
    pub fn since(self, other: Time) -> Duration { Duration(self.0 - other.0) }
    /// Parse "YYYY-MM-DD" or "YYYY-MM-DDTHH:MM:SSZ" (UTC, proleptic Gregorian). No dependency.
    pub fn parse(s: &str) -> Result<Time, String> {
        let (date, clock) = match s.split_once('T') {
            Some((d, c)) => (d, Some(c.trim_end_matches('Z'))),
            None => (s, None),
        };
        let mut it = date.split('-');
        let y: i64 = it.next().ok_or("year")?.parse().map_err(|_| "year")?;
        let m: i64 = it.next().ok_or("month")?.parse().map_err(|_| "month")?;
        let d: i64 = it.next().ok_or("day")?.parse().map_err(|_| "day")?;
        let mut secs = days_from_civil(y, m, d) * 86_400;
        if let Some(c) = clock {
            let mut it = c.split(':');
            let hh: i64 = it.next().ok_or("hour")?.parse().map_err(|_| "hour")?;
            let mm: i64 = it.next().unwrap_or("0").parse().map_err(|_| "minute")?;
            let ss: i64 = it.next().unwrap_or("0").parse().map_err(|_| "second")?;
            secs += hh * 3600 + mm * 60 + ss;
        }
        Ok(Time(secs))
    }
    pub fn to_iso(self) -> String {
        let days = self.0.div_euclid(86_400);
        let rem = self.0.rem_euclid(86_400);
        let (y, m, d) = civil_from_days(days);
        format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, rem / 3600, (rem % 3600) / 60, rem % 60)
    }
}

impl Duration {
    pub const SECOND: Duration = Duration(1);
    pub const MINUTE: Duration = Duration(60);
    pub const HOUR: Duration = Duration(3600);
    pub const DAY: Duration = Duration(86_400);
    pub fn secs(s: i64) -> Self { Duration(s) }
    pub fn hours(h: i64) -> Self { Duration(h * 3600) }
    pub fn days(d: i64) -> Self { Duration(d * 86_400) }
    pub fn as_secs_f64(self) -> f64 { self.0 as f64 }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.to_iso()) }
}

// Howard Hinnant's algorithms, public domain.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn epoch_roundtrip() {
        assert_eq!(Time::parse("1970-01-01").unwrap(), Time(0));
        assert_eq!(Time::parse("2000-03-01T12:00:00Z").unwrap().to_iso(), "2000-03-01T12:00:00Z");
        assert_eq!(Time::parse("1990-01-01").unwrap(), Time(631_152_000));
    }
}
