use super::{
    dates, error, finish, finite, value, DatedAmount, FinanceRequest, FinanceResponse,
    FinanceResult,
};
use std::collections::BTreeMap;

const MIN_RATE: f64 = -0.999999;
const MAX_RATE: f64 = 1_000_000.0;

// Log-scaled positive and negative PV sums avoid overflow over long date spans.
// The returned residual is NPV / gross absolute PV, so scaling does not alter its sign.
fn residual(log_rate: f64, flows: &[(f64, f64)]) -> f64 {
    let largest = flows
        .iter()
        .map(|(amount, years)| amount.abs().ln() - log_rate * years)
        .fold(f64::NEG_INFINITY, f64::max);
    let mut positive = 0.0;
    let mut negative = 0.0;
    for (amount, years) in flows {
        let pv = (amount.abs().ln() - log_rate * years - largest).exp();
        if *amount > 0.0 {
            positive += pv;
        } else {
            negative += pv;
        }
    }
    (positive - negative) / (positive + negative)
}

pub(super) fn calculate(request: &FinanceRequest) -> FinanceResult<FinanceResponse> {
    let FinanceRequest::Xirr { flows } = request else {
        unreachable!()
    };
    if !(2..=200).contains(&flows.len()) {
        return Err(error("cashFlowCount", "cashFlows"));
    }
    let mut dated = BTreeMap::new();
    for (index, flow) in flows.iter().enumerate() {
        let row_error = |mut err: super::FinanceError| {
            err.row = Some(index + 1);
            err
        };
        let date = dates::parse(&flow.date, "cashFlowDate").map_err(row_error)?;
        finite(flow.amount, "cashFlowAmount").map_err(row_error)?;
        *dated.entry(date).or_insert(0.0) += flow.amount;
    }
    dated.retain(|_, amount| *amount != 0.0);
    if dated.len() < 2 {
        return Err(error("cashFlowDates", "cashFlows"));
    }
    let mut changes = 0;
    let mut previous_sign = dated.first_key_value().unwrap().1.is_sign_positive();
    for amount in dated.values().skip(1) {
        let sign = amount.is_sign_positive();
        if sign != previous_sign {
            changes += 1;
        }
        previous_sign = sign;
    }
    if changes == 0 {
        return Err(error("cashFlowSigns", "cashFlows"));
    }
    if changes > 1 {
        return Err(error("cashFlowPattern", "cashFlows"));
    }
    let start = *dated.first_key_value().unwrap().0;
    let end = *dated.last_key_value().unwrap().0;
    let timed: Vec<_> = dated
        .iter()
        .map(|(date, amount)| (*amount, (*date - start).num_days() as f64 / 365.0))
        .collect();
    let mut low = MIN_RATE.ln_1p();
    let mut high = MAX_RATE.ln_1p();
    let low_residual = residual(low, &timed);
    let high_residual = residual(high, &timed);
    if low_residual.signum() == high_residual.signum()
        && low_residual != 0.0
        && high_residual != 0.0
    {
        return Err(error("xirrRange", "cashFlows"));
    }
    for _ in 0..180 {
        let mid = (low + high) * 0.5;
        if residual(mid, &timed).signum() == low_residual.signum() {
            low = mid;
        } else {
            high = mid;
        }
    }
    let rate = ((low + high) * 0.5).exp_m1();
    let check = residual(rate.ln_1p(), &timed);
    if !rate.is_finite() || check.abs() > 1e-10 {
        return Err(error("noConvergence", "cashFlows"));
    }
    let mut response = finish(vec![value("xirrRate", rate, "percent"),
        value("cashInTotal", flows.iter().filter(|f| f.amount > 0.0).map(|f| f.amount).sum(), "amount"),
        value("cashOutTotal", -flows.iter().filter(|f| f.amount < 0.0).map(|f| f.amount).sum::<f64>(), "amount"),
        value("cashNetTotal", flows.iter().map(|f| f.amount).sum(), "amount"),
        value("cashFlowDays", (end - start).num_days() as f64, "days"),
        value("cashFlowEntries", flows.len() as f64, "entries"), value("xirrResidual", check, "ratio")], vec![],
        "Σ CF_i/(1+r)^((date_i−firstEffectiveDate)/365)=0; dates sorted, same-day flows netted; one sign change; effective annual XIRR, not nominal APR; residual=NPV/grossAbsolutePV")?;
    response.dated_cash_flows = Some(
        dated
            .iter()
            .map(|(date, amount)| DatedAmount {
                date: date.to_string(),
                amount: *amount,
            })
            .collect(),
    );
    Ok(response)
}
