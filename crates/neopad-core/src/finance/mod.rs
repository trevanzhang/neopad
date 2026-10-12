//! Offline financial estimates. Rates are fractions, dates are calendar dates,
//! and this module never initializes or writes a workspace.
mod bond;
mod dates;
mod holding;
mod loan;
mod repo;
mod single_bond;
mod xirr;

use serde::{Deserialize, Serialize};

pub const ENGINE_VERSION: &str = "finance-4";
pub type FinanceResult<T> = Result<T, FinanceError>;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FinanceError {
    pub code: &'static str,
    pub field: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row: Option<usize>,
}

pub(crate) fn error(code: &'static str, field: &'static str) -> FinanceError {
    FinanceError {
        code,
        field,
        row: None,
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DayBasis {
    Act365F,
    Act360,
    ActActIsda,
    Us30_360,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RateKind {
    Nominal,
    Effective,
    Periodic,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TvmTarget {
    PresentValue,
    FutureValue,
    Payment,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BondBasis {
    ActActCoupon,
    Us30_360,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BondTarget {
    Price,
    Yield,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SingleBondKind {
    Discount,
    MaturityInterest,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum LoanTarget {
    Payment,
    Principal,
    Rate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum LoanMethod {
    Annuity,
    EqualPrincipal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RepoSide {
    Financing,
    Lending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum FinanceRequest {
    HoldingReturn {
        buy_date: String,
        sell_date: String,
        face: f64,
        buy_price: f64,
        sell_price: f64,
        buy_clean: bool,
        sell_clean: bool,
        buy_accrued: f64,
        sell_accrued: f64,
        coupons: f64,
        buy_fee: f64,
        sell_fee: f64,
        financing_cost: f64,
    },
    Xirr {
        flows: Vec<DatedAmount>,
    },
    Loan {
        target: LoanTarget,
        method: LoanMethod,
        principal: f64,
        rate: f64,
        payment: f64,
        months: u32,
        frequency: u32,
    },
    Repo {
        side: RepoSide,
        amount: f64,
        rate: f64,
        fee_rate: f64,
        fee_at_maturity: bool,
        days: u32,
        start: Option<String>,
        end: Option<String>,
        basis: u32,
    },
    Simple {
        principal: f64,
        rate: f64,
        years: f64,
    },
    DatedSimple {
        principal: f64,
        rate: f64,
        start: String,
        end: String,
        basis: DayBasis,
    },
    Compound {
        principal: f64,
        rate: f64,
        periods: u32,
        compounding: u32,
    },
    Rate {
        rate: f64,
        source: RateKind,
        compounding: u32,
        payments: u32,
    },
    Tvm {
        target: TvmTarget,
        rate: f64,
        compounding: u32,
        payments: u32,
        periods: u32,
        beginning: bool,
        present_value: f64,
        future_value: f64,
        payment: f64,
    },
    Bond {
        target: BondTarget,
        settlement: String,
        maturity: String,
        #[serde(default)]
        previous_coupon: String,
        #[serde(default)]
        next_coupon: String,
        frequency: u32,
        end_of_month: bool,
        basis: BondBasis,
        coupon: f64,
        redemption: f64,
        yield_rate: f64,
        price: f64,
        dirty_input: bool,
        holding: f64,
    },
    SingleBond {
        target: BondTarget,
        profile: SingleBondKind,
        settlement: String,
        maturity: String,
        issue: String,
        basis: DayBasis,
        coupon: f64,
        redemption: f64,
        yield_rate: f64,
        price: f64,
        dirty_input: bool,
        holding: f64,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceValue {
    pub key: &'static str,
    pub value: f64,
    pub unit: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceResponse {
    pub engine_version: &'static str,
    pub values: Vec<FinanceValue>,
    pub cash_flows: Vec<CashFlow>,
    pub formula: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<BondSchedule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repayments: Option<Vec<LoanRepayment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dated_cash_flows: Option<Vec<DatedAmount>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatedAmount {
    pub date: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoanRepayment {
    pub period: u32,
    pub payment: f64,
    pub principal: f64,
    pub interest: f64,
    pub balance: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BondSchedule {
    pub previous_coupon: Option<String>,
    pub next_coupon: String,
    pub remaining_payments: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CashFlow {
    pub date: String,
    pub amount: f64,
    pub years: f64,
    pub present_value: f64,
}

pub(crate) fn value(key: &'static str, value: f64, unit: &'static str) -> FinanceValue {
    FinanceValue { key, value, unit }
}

pub(crate) fn finite(value: f64, field: &'static str) -> FinanceResult<()> {
    if !value.is_finite() || value.abs() > 1e15 {
        return Err(error("range", field));
    }
    Ok(())
}

fn amount(value: f64, field: &'static str) -> FinanceResult<()> {
    finite(value, field)?;
    if value < 0.0 {
        return Err(error("nonnegative", field));
    }
    Ok(())
}

fn frequency(value: u32, field: &'static str) -> FinanceResult<()> {
    if value == 0 || value > 366 {
        return Err(error("frequency", field));
    }
    Ok(())
}

fn periods(value: u32) -> FinanceResult<()> {
    if value == 0 || value > 100_000 {
        return Err(error("periods", "periods"));
    }
    Ok(())
}

fn periodic_rate(rate: f64, compounding: u32, payments: u32) -> FinanceResult<f64> {
    finite(rate, "rate")?;
    frequency(compounding, "compounding")?;
    frequency(payments, "payments")?;
    let base = rate / f64::from(compounding);
    if base <= -1.0 {
        return Err(error("rateDomain", "rate"));
    }
    let result = (base.ln_1p() * f64::from(compounding) / f64::from(payments)).exp_m1();
    if !result.is_finite() || result <= -1.0 {
        return Err(error("overflow", "rate"));
    }
    Ok(result)
}

fn simple(principal: f64, rate: f64, years: f64) -> FinanceResult<Vec<FinanceValue>> {
    amount(principal, "principal")?;
    finite(rate, "rate")?;
    amount(years, "years")?;
    Ok(vec![
        value("interest", principal * rate * years, "amount"),
        value("futureValue", principal * (1.0 + rate * years), "amount"),
        value("years", years, "years"),
    ])
}

pub fn calculate(request: &FinanceRequest) -> FinanceResult<FinanceResponse> {
    let (values, formula) = match request {
        FinanceRequest::HoldingReturn { .. } => return holding::calculate(request),
        FinanceRequest::Xirr { .. } => return xirr::calculate(request),
        FinanceRequest::Loan { .. } => return loan::calculate(request),
        FinanceRequest::Repo { .. } => return repo::calculate(request),
        FinanceRequest::Simple {
            principal,
            rate,
            years,
        } => (
            simple(*principal, *rate, *years)?,
            "I = P × r × t; FV = P + I",
        ),
        FinanceRequest::DatedSimple {
            principal,
            rate,
            start,
            end,
            basis,
        } => {
            let start = dates::parse(start, "start")?;
            let end = dates::parse(end, "end")?;
            let years = dates::year_fraction(start, end, *basis)?;
            let mut values = simple(*principal, *rate, years)?;
            values.push(value("days", (end - start).num_days() as f64, "days"));
            (values, "I = P × r × yearFraction(start, end); [start, end)")
        }
        FinanceRequest::Compound {
            principal,
            rate,
            periods: count,
            compounding,
        } => {
            amount(*principal, "principal")?;
            periods(*count)?;
            let i = periodic_rate(*rate, *compounding, *compounding)?;
            let growth = (f64::from(*count) * i.ln_1p()).exp_m1();
            (
                vec![
                    value("futureValue", principal * (1.0 + growth), "amount"),
                    value("interest", principal * growth, "amount"),
                    value("periodicRate", i, "percent"),
                    value(
                        "years",
                        f64::from(*count) / f64::from(*compounding),
                        "years",
                    ),
                ],
                "FV = P × (1 + j/m)^n; t = n/m",
            )
        }
        FinanceRequest::Rate {
            rate,
            source,
            compounding,
            payments,
        } => {
            finite(*rate, "rate")?;
            frequency(*compounding, "compounding")?;
            frequency(*payments, "payments")?;
            let log_annual = match source {
                RateKind::Nominal => {
                    if rate / f64::from(*compounding) <= -1.0 {
                        return Err(error("rateDomain", "rate"));
                    }
                    (rate / f64::from(*compounding)).ln_1p() * f64::from(*compounding)
                }
                RateKind::Effective => {
                    if *rate <= -1.0 {
                        return Err(error("rateDomain", "rate"));
                    }
                    rate.ln_1p()
                }
                RateKind::Periodic => {
                    if *rate <= -1.0 {
                        return Err(error("rateDomain", "rate"));
                    }
                    rate.ln_1p() * f64::from(*payments)
                }
            };
            (
                vec![
                    value("effectiveRate", log_annual.exp_m1(), "percent"),
                    value(
                        "nominalRate",
                        (log_annual / f64::from(*compounding)).exp_m1() * f64::from(*compounding),
                        "percent",
                    ),
                    value(
                        "periodicRate",
                        (log_annual / f64::from(*payments)).exp_m1(),
                        "percent",
                    ),
                ],
                "EAR = (1+j/m)^m−1; i = (1+EAR)^(1/p)−1",
            )
        }
        FinanceRequest::Tvm {
            target,
            rate,
            compounding,
            payments,
            periods: count,
            beginning,
            present_value,
            future_value,
            payment,
        } => {
            periods(*count)?;
            finite(*present_value, "presentValue")?;
            finite(*future_value, "futureValue")?;
            finite(*payment, "payment")?;
            let i = periodic_rate(*rate, *compounding, *payments)?;
            let growth = (f64::from(*count) * i.ln_1p()).exp();
            if !growth.is_finite() || growth == 0.0 {
                return Err(error("overflow", "periods"));
            }
            let annuity = if i == 0.0 {
                f64::from(*count)
            } else {
                (f64::from(*count) * i.ln_1p()).exp_m1() / i
            } * if *beginning { 1.0 + i } else { 1.0 };
            if !annuity.is_finite() || annuity <= 0.0 {
                return Err(error("overflow", "periods"));
            }
            let (key, result) = match target {
                TvmTarget::PresentValue => {
                    ("presentValue", -(payment * annuity + future_value) / growth)
                }
                TvmTarget::FutureValue => {
                    ("futureValue", -(present_value * growth + payment * annuity))
                }
                TvmTarget::Payment => (
                    "payment",
                    -(present_value * growth + future_value) / annuity,
                ),
            };
            (
                vec![
                    value(key, result, "amount"),
                    value("periodicRate", i, "percent"),
                ],
                "PV × (1+i)^n + PMT × (1+i×b) × ((1+i)^n−1)/i + FV = 0",
            )
        }
        FinanceRequest::Bond { .. } => return bond::calculate(request),
        FinanceRequest::SingleBond { .. } => return single_bond::calculate(request),
    };
    finish(values, Vec::new(), formula)
}

pub(crate) fn finish(
    values: Vec<FinanceValue>,
    cash_flows: Vec<CashFlow>,
    formula: &'static str,
) -> FinanceResult<FinanceResponse> {
    if values.iter().any(|v| !v.value.is_finite())
        || cash_flows
            .iter()
            .any(|v| !v.present_value.is_finite() || !v.years.is_finite() || !v.amount.is_finite())
    {
        return Err(error("overflow", "result"));
    }
    Ok(FinanceResponse {
        engine_version: ENGINE_VERSION,
        values,
        cash_flows,
        formula,
        schedule: None,
        repayments: None,
        dated_cash_flows: None,
    })
}

#[cfg(test)]
mod return_tests;
#[cfg(test)]
mod tests;
