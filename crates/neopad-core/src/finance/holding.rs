use super::{dates, error, finish, finite, value, FinanceRequest, FinanceResponse, FinanceResult};

pub(super) fn calculate(request: &FinanceRequest) -> FinanceResult<FinanceResponse> {
    let FinanceRequest::HoldingReturn {
        buy_date,
        sell_date,
        face,
        buy_price,
        sell_price,
        buy_clean,
        sell_clean,
        buy_accrued,
        sell_accrued,
        coupons,
        buy_fee,
        sell_fee,
        financing_cost,
    } = request
    else {
        unreachable!()
    };
    let start = dates::parse(buy_date, "holdingBuyDate")?;
    let end = dates::parse(sell_date, "holdingSellDate")?;
    if end <= start {
        return Err(error("dateOrder", "holdingSellDate"));
    }
    for (n, field, strictly_positive) in [
        (*face, "holdingFace", true),
        (*buy_price, "holdingBuyPrice", true),
        (*sell_price, "holdingSellPrice", false),
        (*coupons, "holdingCoupons", false),
        (*buy_fee, "holdingBuyFee", false),
        (*sell_fee, "holdingSellFee", false),
        (*financing_cost, "holdingFinancingCost", false),
    ] {
        finite(n, field)?;
        if n < 0.0 || (strictly_positive && n == 0.0) {
            return Err(error(
                if strictly_positive {
                    "positive"
                } else {
                    "nonnegative"
                },
                field,
            ));
        }
    }
    // Accrued interest is relevant only when converting clean quotes to full prices.
    for (clean, n, field) in [
        (*buy_clean, *buy_accrued, "holdingBuyAccrued"),
        (*sell_clean, *sell_accrued, "holdingSellAccrued"),
    ] {
        if clean {
            finite(n, field)?;
            if n < 0.0 {
                return Err(error("nonnegative", field));
            }
        }
    }
    let buy_full = buy_price + if *buy_clean { *buy_accrued } else { 0.0 };
    let sell_full = sell_price + if *sell_clean { *sell_accrued } else { 0.0 };
    let buy_amount = face / 100.0 * buy_full;
    let sell_amount = face / 100.0 * sell_full;
    let initial = buy_amount + buy_fee;
    let gain = sell_amount - buy_amount;
    let fees = buy_fee + sell_fee;
    let profit = gain + coupons - fees - financing_cost;
    let days = (end - start).num_days() as f64;
    finish(vec![value("holdingBuyFull", buy_full, "price"), value("holdingSellFull", sell_full, "price"),
        value("holdingBuyAmount", buy_amount, "amount"), value("holdingSellAmount", sell_amount, "amount"),
        value("holdingInitial", initial, "amount"), value("holdingPriceGain", gain, "amount"),
        value("holdingCoupons", *coupons, "amount"), value("holdingFees", fees, "amount"),
        value("holdingFinancingCost", *financing_cost, "amount"), value("holdingProfit", profit, "amount"),
        value("holdingReturn", profit / initial, "percent"), value("holdingAnnualSimple", profit / initial * 365.0 / days, "percent"),
        value("holdingDays", days, "days")], vec![],
        "fullPrice=cleanPrice+accruedInterest; buy/sellAmount=fullPrice×face/100; profit=sellAmount−buyAmount+coupons−buyFee−sellFee−financingCost; HPR=profit/(buyAmount+buyFee); simpleAnnual=HPR×365/actualDays; no coupon reinvestment or leverage ROE")
}
