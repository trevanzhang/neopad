use super::{
    dates, error, finish, finite, value, FinanceRequest, FinanceResponse, FinanceResult, RepoSide,
};

pub(super) fn calculate(request: &FinanceRequest) -> FinanceResult<FinanceResponse> {
    let FinanceRequest::Repo {
        side,
        amount,
        rate,
        fee_rate,
        fee_at_maturity,
        days,
        start,
        end,
        basis,
    } = request
    else {
        unreachable!()
    };
    for (n, field) in [
        (*amount, "repoAmount"),
        (*rate, "repoRate"),
        (*fee_rate, "repoFeeRate"),
    ] {
        finite(n, field)?;
    }
    if *amount <= 0.0 {
        return Err(error("positive", "repoAmount"));
    }
    if *rate < 0.0 {
        return Err(error("nonnegative", "repoRate"));
    }
    if !(0.0..=1.0).contains(fee_rate) {
        return Err(error("repoFeeRange", "repoFeeRate"));
    }
    if ![360, 365].contains(basis) {
        return Err(error("repoBasis", "repoBasis"));
    }
    let days = match (start, end) {
        (Some(start), Some(end)) => {
            let start = dates::parse(start, "repoStart")?;
            let end = dates::parse(end, "repoEnd")?;
            if end <= start {
                return Err(error("dateOrder", "repoEnd"));
            }
            (end - start).num_days() as u32
        }
        (None, None) => *days,
        _ => return Err(error("date", "repoStart")),
    };
    if days == 0 || days > 36600 {
        return Err(error("repoDaysRange", "repoDays"));
    }
    let interest = amount * rate * f64::from(days) / f64::from(*basis);
    let fee = amount * fee_rate;
    let lending = *side == RepoSide::Lending;
    let net = if lending {
        interest - fee
    } else {
        interest + fee
    };
    finish(vec![
        value("repoInterest", interest, "amount"), value("repoFee", fee, "amount"),
        value(if lending { "repoNetIncome" } else { "repoFinancingCost" }, net, "amount"),
        value(if lending { "repoNetAnnualRate" } else { "repoCostAnnualRate" }, net / amount * f64::from(*basis) / f64::from(days), "percent"),
        value("repoInitialFlow", (if lending { -amount } else { *amount }) - if *fee_at_maturity { 0.0 } else { fee }, "amount"),
        value("repoMaturityFlow", (if lending { amount + interest } else { -amount - interest }) - if *fee_at_maturity { fee } else { 0.0 }, "amount"),
        value("repoDays", f64::from(days), "days"),
    ], vec![], "I=P×r×actualDays/B; fee=P×feeRate (once, at selected timing); lendingNet=I−fee; financingCost=I+fee; netAnnualRate=net/P×B/actualDays (not IRR)")
}
