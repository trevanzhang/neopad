use super::{
    dates, error, finish, finite, value, BondSchedule, BondTarget, CashFlow, FinanceRequest,
    FinanceResponse, FinanceResult, SingleBondKind,
};

/// Single repayment, simple annual investment yield (not bank discount yield).
pub(super) fn calculate(request: &FinanceRequest) -> FinanceResult<FinanceResponse> {
    let FinanceRequest::SingleBond {
        target,
        profile,
        settlement,
        maturity,
        issue,
        basis,
        coupon,
        redemption,
        yield_rate,
        price,
        dirty_input,
        holding,
    } = request
    else {
        unreachable!()
    };
    for (n, field) in [(*redemption, "redemption"), (*holding, "holding")] {
        finite(n, field)?;
    }
    if *redemption <= 0.0 {
        return Err(error("positive", "redemption"));
    }
    if *holding < 0.0 {
        return Err(error("nonnegative", "holding"));
    }
    let settlement = dates::parse(settlement, "settlement")?;
    let maturity = dates::parse(maturity, "maturity")?;
    if maturity <= settlement {
        return Err(error("dateOrder", "maturity"));
    }
    let years = dates::year_fraction(settlement, maturity, *basis)?;
    if years <= 0.0 {
        return Err(error("range", "basis"));
    }
    let (repayment, accrued) = if *profile == SingleBondKind::MaturityInterest {
        finite(*coupon, "coupon")?;
        if *coupon < 0.0 {
            return Err(error("nonnegative", "coupon"));
        }
        let issue = dates::parse(issue, "issue")?;
        if issue > settlement {
            return Err(error("dateOrder", "issue"));
        }
        let accrued_years = if issue == settlement {
            0.0
        } else {
            dates::year_fraction(issue, settlement, *basis)?
        };
        (
            redemption + 100.0 * coupon * dates::year_fraction(issue, maturity, *basis)?,
            100.0 * coupon * accrued_years,
        )
    } else {
        (*redemption, 0.0)
    };
    let (y, full) = match target {
        BondTarget::Price => {
            finite(*yield_rate, "yieldRate")?;
            let denominator = 1.0 + yield_rate * years;
            if denominator <= 0.0 {
                return Err(error("rateDomain", "yieldRate"));
            }
            (*yield_rate, repayment / denominator)
        }
        BondTarget::Yield => {
            finite(*price, "price")?;
            if *price <= 0.0 {
                return Err(error("positive", "price"));
            }
            let full = price + if *dirty_input { 0.0 } else { accrued };
            ((repayment / full - 1.0) / years, full)
        }
    };
    let denominator = 1.0 + y * years;
    let duration = years / denominator;
    let dv01 = full * duration * 0.0001;
    let mut result = finish(vec![
        value("cleanPrice", full - accrued, "price"), value("dirtyPrice", full, "price"),
        value("accruedInterest", accrued, "price"), value("yieldRate", y, "percent"),
        value("macaulayDuration", years, "years"), value("modifiedDuration", duration, "years"),
        value("convexity", 2.0 * duration.powi(2), "yearsSquared"), value("dv01", dv01, "price"),
        value("holdingDv01", dv01 * holding / 100.0, "amount"),
        value("holdingValue", full * holding / 100.0, "amount"),
        value("residual", repayment / denominator - full, "price"),
    ], vec![CashFlow { date: maturity.to_string(), amount: repayment, years, present_value: full }],
    "Pfull = repayment/(1+y×t); Pclean = Pfull−AI; y = (repayment/Pfull−1)/t; Dmod = t/(1+y×t); convexity = 2×Dmod²")?;
    result.schedule = Some(BondSchedule {
        previous_coupon: None,
        next_coupon: maturity.to_string(),
        remaining_payments: 1,
    });
    Ok(result)
}
