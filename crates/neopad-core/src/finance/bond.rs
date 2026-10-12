use super::{
    dates, error, finish, finite, value, BondBasis, BondSchedule, BondTarget, CashFlow,
    FinanceRequest, FinanceResponse, FinanceResult,
};
use chrono::{Datelike, NaiveDate};

struct Schedule {
    dates: Vec<NaiveDate>,
    previous: NaiveDate,
    fraction: f64,
    accrued: f64,
}

fn coupon_date(
    maturity: NaiveDate,
    months_back: i32,
    end_of_month: bool,
) -> FinanceResult<NaiveDate> {
    let months = maturity.year() * 12 + maturity.month0() as i32 - months_back;
    let year = months.div_euclid(12);
    let month = months.rem_euclid(12) as u32 + 1;
    if year < 1600 {
        return Err(error("range", "maturity"));
    }
    let day = if end_of_month {
        dates::month_end(year, month)
    } else {
        maturity.day().min(dates::month_end(year, month))
    };
    NaiveDate::from_ymd_opt(year, month, day).ok_or(error("date", "maturity"))
}

fn schedule(
    settlement: NaiveDate,
    maturity: NaiveDate,
    previous: Option<NaiveDate>,
    next: Option<NaiveDate>,
    frequency: u32,
    end_of_month: bool,
    basis: BondBasis,
    coupon: f64,
) -> FinanceResult<Schedule> {
    if settlement >= maturity {
        return Err(error("dateOrder", "maturity"));
    }
    if ![1, 2, 3, 4].contains(&frequency) {
        return Err(error("bondFrequency", "frequency"));
    }
    if end_of_month && maturity.day() != dates::month_end(maturity.year(), maturity.month()) {
        return Err(error("schedule", "endOfMonth"));
    }
    let mut upcoming = Vec::new();
    let step = 12 / frequency as i32;
    for index in 0..=500 {
        let date = coupon_date(maturity, index * step, end_of_month)?;
        if date <= settlement {
            if previous.is_some_and(|previous| date != previous) {
                return Err(error("schedule", "previousCoupon"));
            }
            let actual_next = *upcoming.last().ok_or(error("schedule", "nextCoupon"))?;
            if next.is_some_and(|next| next != actual_next) {
                return Err(error("schedule", "nextCoupon"));
            }
            let previous = date;
            let next = actual_next;
            upcoming.reverse();
            let (accrued_days, remaining_days, period_days) = match basis {
                BondBasis::ActActCoupon => (
                    (settlement - previous).num_days() as f64,
                    (next - settlement).num_days() as f64,
                    (next - previous).num_days() as f64,
                ),
                BondBasis::Us30_360 => (
                    dates::nasd_days(previous, settlement),
                    dates::nasd_days(settlement, next),
                    360.0 / f64::from(frequency),
                ),
            };
            if remaining_days <= 0.0 || period_days <= 0.0 {
                return Err(error("range", "basis"));
            }
            return Ok(Schedule {
                dates: upcoming,
                previous,
                fraction: remaining_days / period_days,
                accrued: 100.0 * coupon / f64::from(frequency) * accrued_days / period_days,
            });
        }
        upcoming.push(date);
    }
    Err(error("range", "maturity"))
}

// Full price and its first/second derivatives with respect to annual yield.
fn price(
    schedule: &Schedule,
    coupon: f64,
    redemption: f64,
    frequency: u32,
    yield_rate: f64,
) -> FinanceResult<(f64, f64, f64, Vec<CashFlow>)> {
    let f = f64::from(frequency);
    let base = 1.0 + yield_rate / f;
    if base <= 0.0 && schedule.dates.len() != 1 {
        return Err(error("rateDomain", "yieldRate"));
    }
    let last_period = schedule.dates.len() == 1;
    let mut full = 0.0;
    let mut derivative = 0.0;
    let mut curvature = 0.0;
    let mut flows = Vec::with_capacity(schedule.dates.len());
    for (index, date) in schedule.dates.iter().enumerate() {
        let periods = schedule.fraction + index as f64;
        let years = periods / f;
        let amount = 100.0 * coupon / f
            + if index + 1 == schedule.dates.len() {
                redemption
            } else {
                0.0
            };
        let (pv, slope, second) = if last_period {
            let denominator = 1.0 + yield_rate * years;
            if denominator <= 0.0 {
                return Err(error("rateDomain", "yieldRate"));
            }
            (
                amount / denominator,
                -amount * years / denominator.powi(2),
                2.0 * amount * years.powi(2) / denominator.powi(3),
            )
        } else {
            let pv = amount / (periods * (yield_rate / f).ln_1p()).exp();
            (
                pv,
                -pv * years / base,
                pv * years * (years + 1.0 / f) / base.powi(2),
            )
        };
        full += pv;
        derivative += slope;
        curvature += second;
        flows.push(CashFlow {
            date: date.to_string(),
            amount,
            years,
            present_value: pv,
        });
    }
    if !full.is_finite() || full <= 0.0 || !derivative.is_finite() || !curvature.is_finite() {
        return Err(error("overflow", "yieldRate"));
    }
    Ok((full, derivative, curvature, flows))
}

fn solve_yield(
    schedule: &Schedule,
    coupon: f64,
    redemption: f64,
    frequency: u32,
    target: f64,
) -> FinanceResult<(f64, f64)> {
    if !target.is_finite() || target <= 0.0 {
        return Err(error("positive", "price"));
    }
    let at_zero = price(schedule, coupon, redemption, frequency, 0.0)?.0;
    if (at_zero - target).abs() <= 1e-10 {
        return Ok((0.0, at_zero - target));
    }
    // Positive cash flows make price strictly decreasing. Approach the pole
    // adaptively; do not price directly at -frequency (overflow for long bonds).
    let pole = if schedule.dates.len() == 1 {
        -f64::from(frequency) / schedule.fraction
    } else {
        -f64::from(frequency)
    };
    let mut low = 0.0;
    if target > at_zero {
        low = pole * 0.5;
        for _ in 0..60 {
            match price(schedule, coupon, redemption, frequency, low) {
                Ok((p, ..)) if p < target => low = (low + pole) * 0.5,
                _ => break,
            }
        }
    }
    let mut high = 1.0;
    while price(schedule, coupon, redemption, frequency, high)?.0 > target && high < 1024.0 {
        high *= 2.0;
    }
    if price(schedule, coupon, redemption, frequency, high)?.0 > target {
        return Err(error("noConvergence", "price"));
    }
    for _ in 0..160 {
        let mid = (low + high) / 2.0;
        let residual = price(schedule, coupon, redemption, frequency, mid)?.0 - target;
        if residual.abs() <= 1e-10 {
            return Ok((mid, residual));
        }
        if residual > 0.0 {
            low = mid;
        } else {
            high = mid;
        }
    }
    Err(error("noConvergence", "price"))
}

pub(super) fn calculate(request: &FinanceRequest) -> FinanceResult<FinanceResponse> {
    let FinanceRequest::Bond {
        target,
        settlement,
        maturity,
        previous_coupon,
        next_coupon,
        frequency,
        end_of_month,
        basis,
        coupon,
        redemption,
        yield_rate,
        price: input_price,
        dirty_input,
        holding,
    } = request
    else {
        unreachable!()
    };
    for (number, field) in [
        (*coupon, "coupon"),
        (*redemption, "redemption"),
        (*holding, "holding"),
    ] {
        finite(number, field)?;
    }
    if *coupon <= 0.0 {
        return Err(error("positive", "coupon"));
    }
    if *redemption <= 0.0 {
        return Err(error("positive", "redemption"));
    }
    if *holding < 0.0 {
        return Err(error("nonnegative", "holding"));
    }
    let schedule = schedule(
        dates::parse(settlement, "settlement")?,
        dates::parse(maturity, "maturity")?,
        if previous_coupon.is_empty() {
            None
        } else {
            Some(dates::parse(previous_coupon, "previousCoupon")?)
        },
        if next_coupon.is_empty() {
            None
        } else {
            Some(dates::parse(next_coupon, "nextCoupon")?)
        },
        *frequency,
        *end_of_month,
        *basis,
        *coupon,
    )?;
    let (y, residual) = match target {
        BondTarget::Price => {
            finite(*yield_rate, "yieldRate")?;
            (*yield_rate, 0.0)
        }
        BondTarget::Yield => {
            finite(*input_price, "price")?;
            if *input_price <= 0.0 {
                return Err(error("positive", "price"));
            }
            solve_yield(
                &schedule,
                *coupon,
                *redemption,
                *frequency,
                input_price + if *dirty_input { 0.0 } else { schedule.accrued },
            )?
        }
    };
    let (full, derivative, curvature, flows) =
        price(&schedule, *coupon, *redemption, *frequency, y)?;
    let macaulay = flows.iter().map(|c| c.years * c.present_value).sum::<f64>() / full;
    let dv01 = -derivative * 0.0001;
    let mut response = finish(
        vec![
            value("cleanPrice", full - schedule.accrued, "price"),
            value("dirtyPrice", full, "price"),
            value("accruedInterest", schedule.accrued, "price"),
            value("yieldRate", y, "percent"),
            value("macaulayDuration", macaulay, "years"),
            value("modifiedDuration", -derivative / full, "years"),
            value("convexity", curvature / full, "yearsSquared"),
            value("dv01", dv01, "price"),
            value("holdingDv01", dv01 * holding / 100.0, "amount"),
            value("holdingValue", full * holding / 100.0, "amount"),
            value("residual", residual, "price"),
        ],
        flows,
        if schedule.dates.len() == 1 {
            "Pfull = (C/f + R) / (1 + y×DSC/(f×E)); Pclean = Pfull−AI; Dmod = −P′/Pfull"
        } else {
            "Pfull = Σ CFk/(1+y/f)^(DSC/E+k); Pclean = Pfull−AI; Dmod = −P′/Pfull"
        },
    )?;
    response.schedule = Some(BondSchedule {
        previous_coupon: Some(schedule.previous.to_string()),
        next_coupon: schedule.dates[0].to_string(),
        remaining_payments: schedule.dates.len(),
    });
    Ok(response)
}
