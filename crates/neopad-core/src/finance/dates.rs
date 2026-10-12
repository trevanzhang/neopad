use super::{error, DayBasis, FinanceResult};
use chrono::{Datelike, NaiveDate};

pub(super) fn parse(input: &str, field: &'static str) -> FinanceResult<NaiveDate> {
    let date = NaiveDate::parse_from_str(input, "%Y-%m-%d").map_err(|_| error("date", field))?;
    if date.format("%Y-%m-%d").to_string() != input || !(1600..=9999).contains(&date.year()) {
        return Err(error("date", field));
    }
    Ok(date)
}

pub(super) fn month_end(year: i32, month: u32) -> u32 {
    for day in (28..=31).rev() {
        if NaiveDate::from_ymd_opt(year, month, day).is_some() {
            return day;
        }
    }
    unreachable!()
}

pub(super) fn year_fraction(
    start: NaiveDate,
    end: NaiveDate,
    basis: DayBasis,
) -> FinanceResult<f64> {
    if end <= start {
        return Err(error("dateOrder", "end"));
    }
    let days = (end - start).num_days() as f64;
    match basis {
        DayBasis::Act365F => Ok(days / 365.0),
        DayBasis::Act360 => Ok(days / 360.0),
        DayBasis::Us30_360 => Ok(nasd_days(start, end) / 360.0),
        DayBasis::ActActIsda => {
            let mut cursor = start;
            let mut years = 0.0;
            while cursor < end {
                let next_year =
                    NaiveDate::from_ymd_opt(cursor.year() + 1, 1, 1).ok_or(error("date", "end"))?;
                let january = NaiveDate::from_ymd_opt(cursor.year(), 1, 1).unwrap();
                let boundary = end.min(next_year);
                years +=
                    (boundary - cursor).num_days() as f64 / (next_year - january).num_days() as f64;
                cursor = boundary;
            }
            Ok(years)
        }
    }
}

/// NASD 30/360, including February month ends. Kept separate from ISDA/European variants.
pub(super) fn nasd_days(start: NaiveDate, end: NaiveDate) -> f64 {
    let start_feb_end = start.month() == 2 && start.day() == month_end(start.year(), 2);
    let end_feb_end = end.month() == 2 && end.day() == month_end(end.year(), 2);
    let d1 = if start.day() == 31 || start_feb_end {
        30
    } else {
        start.day()
    };
    let d2 = if (end.day() == 31 && d1 >= 30) || (start_feb_end && end_feb_end) {
        30
    } else {
        end.day()
    };
    f64::from(
        360 * (end.year() - start.year())
            + 30 * (end.month() as i32 - start.month() as i32)
            + d2 as i32
            - d1 as i32,
    )
}
