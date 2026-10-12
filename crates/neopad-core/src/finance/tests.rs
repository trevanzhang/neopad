use super::*;

fn loan_request(
    method: LoanMethod,
    principal: f64,
    rate: f64,
    months: u32,
    frequency: u32,
) -> FinanceRequest {
    FinanceRequest::Loan {
        target: LoanTarget::Payment,
        method,
        principal,
        rate,
        payment: 0.0,
        months,
        frequency,
    }
}

#[test]
fn loans_match_independent_pmt_rate_and_equal_principal_cases() {
    // Microsoft PMT and RATE documentation examples; RATE reports a rounded annual rate.
    let pmt = calculate(&loan_request(LoanMethod::Annuity, 10000.0, 0.08, 10, 12)).unwrap();
    close(number(&pmt, "firstPayment"), 1037.03, 0.005);
    let rate = calculate(&FinanceRequest::Loan {
        target: LoanTarget::Rate,
        method: LoanMethod::Annuity,
        principal: 8000.0,
        rate: 0.0,
        payment: 200.0,
        months: 48,
        frequency: 12,
    })
    .unwrap();
    close(number(&rate, "loanRate"), 0.0924, 0.00005);
    let equal = calculate(&loan_request(
        LoanMethod::EqualPrincipal,
        12000.0,
        0.12,
        12,
        12,
    ))
    .unwrap();
    for (key, expected) in [
        ("firstPayment", 1120.0),
        ("lastPayment", 1010.0),
        ("paymentDecrease", 10.0),
        ("totalInterest", 780.0),
        ("totalRepayment", 12780.0),
    ] {
        close(number(&equal, key), expected, 1e-9);
    }
}

#[test]
fn loan_inverse_and_complete_schedule_conserve_principal_and_interest() {
    for method in [LoanMethod::Annuity, LoanMethod::EqualPrincipal] {
        for frequency in [1, 2, 4, 12] {
            for rate in [0.0, 1e-10, 0.035, 1.0, 10.0] {
                let result =
                    calculate(&loan_request(method, 100000.0, rate, 1200, frequency)).unwrap();
                let rows = result.repayments.as_ref().unwrap();
                assert_eq!(rows.len(), 100 * frequency as usize);
                assert_eq!(rows.last().unwrap().balance, 0.0);
                close(rows.iter().map(|r| r.principal).sum(), 100000.0, 1e-8);
                close(
                    rows.iter().map(|r| r.interest).sum(),
                    number(&result, "totalInterest"),
                    1e-8,
                );
                close(
                    rows.iter().map(|r| r.payment).sum(),
                    number(&result, "totalRepayment"),
                    1e-5,
                );
                for row in rows {
                    assert!(row.principal >= 0.0 && row.interest >= 0.0 && row.balance >= 0.0);
                    if method == LoanMethod::Annuity {
                        close(row.payment, rows[0].payment, 1e-8);
                    } else {
                        close(row.principal, rows[0].principal, 1e-8);
                    }
                }
                for target in [LoanTarget::Principal, LoanTarget::Rate] {
                    let inverse = calculate(&FinanceRequest::Loan {
                        target,
                        method,
                        principal: 100000.0,
                        rate,
                        payment: rows[0].payment,
                        months: 1200,
                        frequency,
                    })
                    .unwrap();
                    close(number(&inverse, "loanPrincipal"), 100000.0, 1e-6);
                    close(number(&inverse, "loanRate"), rate, 1e-9);
                }
            }
        }
    }
    let tiny = calculate(&FinanceRequest::Loan {
        target: LoanTarget::Rate,
        method: LoanMethod::Annuity,
        principal: 1e-8,
        rate: 0.0,
        payment: 2e-9,
        months: 12,
        frequency: 12,
    })
    .unwrap();
    assert!(number(&tiny, "loanRate") > 1.0);
}

#[test]
fn loan_rejects_invalid_term_frequency_rate_and_impossible_payment() {
    for (months, frequency, rate, code) in [
        (0, 12, 0.03, "loanTerm"),
        (1201, 12, 0.03, "loanTerm"),
        (13, 4, 0.03, "termFrequency"),
        (12, 3, 0.03, "loanFrequency"),
        (12, 12, -0.03, "loanRateRange"),
        (12, 12, 10.1, "loanRateRange"),
    ] {
        assert_eq!(
            calculate(&loan_request(
                LoanMethod::Annuity,
                1000.0,
                rate,
                months,
                frequency
            ))
            .unwrap_err()
            .code,
            code
        );
    }
    assert!(calculate(&loan_request(LoanMethod::Annuity, 0.0, 0.03, 12, 12)).is_err());
    for method in [LoanMethod::Annuity, LoanMethod::EqualPrincipal] {
        for (payment, code) in [
            (99.0, "loanPaymentLow"),
            (10000.0, "loanRateRange"),
            (0.0, "positive"),
        ] {
            assert_eq!(
                calculate(&FinanceRequest::Loan {
                    target: LoanTarget::Rate,
                    method,
                    principal: 1200.0,
                    rate: 0.0,
                    payment,
                    months: 12,
                    frequency: 12
                })
                .unwrap_err()
                .code,
                code
            );
        }
    }
}

fn repo_request(side: RepoSide, fee_at_maturity: bool) -> FinanceRequest {
    FinanceRequest::Repo {
        side,
        amount: 100000.0,
        rate: 0.02,
        fee_rate: 0.00001,
        fee_at_maturity,
        days: 3,
        start: None,
        end: None,
        basis: 365,
    }
}

#[test]
fn repo_directions_fee_timing_and_actual_dates_preserve_cash_flows() {
    let interest = 100000.0 * 0.02 * 3.0 / 365.0;
    for side in [RepoSide::Lending, RepoSide::Financing] {
        for timing in [false, true] {
            let result = calculate(&repo_request(side, timing)).unwrap();
            close(number(&result, "repoInterest"), interest, 1e-12);
            close(number(&result, "repoFee"), 1.0, 1e-12);
            let sum = number(&result, "repoInitialFlow") + number(&result, "repoMaturityFlow");
            if side == RepoSide::Lending {
                close(number(&result, "repoNetIncome"), interest - 1.0, 1e-12);
                close(sum, interest - 1.0, 1e-10);
                assert!(number(&result, "repoInitialFlow") < 0.0);
            } else {
                close(number(&result, "repoFinancingCost"), interest + 1.0, 1e-12);
                close(sum, -interest - 1.0, 1e-10);
                assert!(number(&result, "repoInitialFlow") > 0.0);
            }
        }
    }
    let dated = calculate(&FinanceRequest::Repo {
        side: RepoSide::Lending,
        amount: 100000.0,
        rate: 0.02,
        fee_rate: 0.0,
        fee_at_maturity: false,
        days: 1,
        start: Some("2026-10-16".into()),
        end: Some("2026-10-19".into()),
        basis: 360,
    })
    .unwrap();
    close(number(&dated, "repoDays"), 3.0, 0.0);
    close(
        number(&dated, "repoInterest"),
        100000.0 * 0.02 * 3.0 / 360.0,
        1e-12,
    );
    let negative = calculate(&FinanceRequest::Repo {
        side: RepoSide::Lending,
        amount: 100000.0,
        rate: 0.0,
        fee_rate: 0.01,
        fee_at_maturity: false,
        days: 1,
        start: None,
        end: None,
        basis: 365,
    })
    .unwrap();
    close(number(&negative, "repoNetIncome"), -1000.0, 1e-12);
    assert!(number(&negative, "repoNetAnnualRate") < 0.0);
}

#[test]
fn repo_rejects_invalid_amount_fee_basis_days_and_dates() {
    for (amount, rate, fee, days, basis) in [
        (0.0, 0.02, 0.0, 1, 365),
        (100.0, -0.02, 0.0, 1, 365),
        (100.0, 0.02, 1.1, 1, 365),
        (100.0, 0.02, -0.1, 1, 365),
        (100.0, 0.02, 0.0, 0, 365),
        (100.0, 0.02, 0.0, 36601, 365),
        (100.0, 0.02, 0.0, 1, 366),
    ] {
        assert!(calculate(&FinanceRequest::Repo {
            side: RepoSide::Financing,
            amount,
            rate,
            fee_rate: fee,
            fee_at_maturity: false,
            days,
            start: None,
            end: None,
            basis
        })
        .is_err());
    }
    for (start, end) in [
        (Some("2026-10-12"), None),
        (Some("2026-10-12"), Some("2026-10-12")),
        (Some("2026-10-13"), Some("2026-10-12")),
        (Some("2026-02-30"), Some("2026-03-02")),
    ] {
        assert!(calculate(&FinanceRequest::Repo {
            side: RepoSide::Lending,
            amount: 100.0,
            rate: 0.02,
            fee_rate: 0.0,
            fee_at_maturity: false,
            days: 1,
            start: start.map(Into::into),
            end: end.map(Into::into),
            basis: 365
        })
        .is_err());
    }
}

#[test]
fn loan_and_repo_wire_format_exposes_rows_and_direction_without_bond_units() {
    let request: FinanceRequest = serde_json::from_str(r#"{"kind":"loan","target":"payment","method":"equalPrincipal","principal":12000,"rate":0.12,"payment":0,"months":12,"frequency":12}"#).unwrap();
    let response = serde_json::to_value(calculate(&request).unwrap()).unwrap();
    assert_eq!(response["repayments"][0]["principal"], 1000.0);
    assert_eq!(response["repayments"][0]["balance"], 11000.0);
    assert_eq!(response["cashFlows"].as_array().unwrap().len(), 0);
    let request: FinanceRequest = serde_json::from_str(r#"{"kind":"repo","side":"financing","amount":100000,"rate":0.02,"feeRate":0.00001,"feeAtMaturity":true,"days":3,"start":null,"end":null,"basis":365}"#).unwrap();
    let response = calculate(&request).unwrap();
    close(number(&response, "repoInitialFlow"), 100000.0, 0.0);
    assert!(serde_json::to_value(response)
        .unwrap()
        .get("repayments")
        .is_none());
}

fn number(response: &FinanceResponse, key: &str) -> f64 {
    response.values.iter().find(|v| v.key == key).unwrap().value
}

fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}

fn bond_request() -> FinanceRequest {
    FinanceRequest::Bond {
        target: BondTarget::Price,
        settlement: "2026-10-01".into(),
        maturity: "2031-10-01".into(),
        previous_coupon: "2026-10-01".into(),
        next_coupon: "2027-04-01".into(),
        frequency: 2,
        end_of_month: false,
        basis: BondBasis::ActActCoupon,
        coupon: 0.06,
        redemption: 100.0,
        yield_rate: 0.08,
        price: 100.0,
        dirty_input: false,
        holding: 1_000_000.0,
    }
}

#[test]
fn money_and_rate_reference_cases() {
    let simple = calculate(&FinanceRequest::Simple {
        principal: 100_000.0,
        rate: 0.03,
        years: 30.0 / 365.0,
    })
    .unwrap();
    close(number(&simple, "interest"), 246.575342465753, 1e-9);
    let compound = calculate(&FinanceRequest::Compound {
        principal: 100_000.0,
        rate: 0.03,
        periods: 2,
        compounding: 1,
    })
    .unwrap();
    close(number(&compound, "futureValue"), 106_090.0, 1e-8);
    let rate = calculate(&FinanceRequest::Rate {
        rate: 0.12,
        source: RateKind::Nominal,
        compounding: 2,
        payments: 12,
    })
    .unwrap();
    close(number(&rate, "effectiveRate"), 0.1236, 1e-12);
    let inverse = calculate(&FinanceRequest::Rate {
        rate: number(&rate, "effectiveRate"),
        source: RateKind::Effective,
        compounding: 2,
        payments: 12,
    })
    .unwrap();
    close(number(&inverse, "nominalRate"), 0.12, 1e-12);
    let microsoft_effect = calculate(&FinanceRequest::Rate {
        rate: 0.0525,
        source: RateKind::Nominal,
        compounding: 4,
        payments: 4,
    })
    .unwrap();
    close(number(&microsoft_effect, "effectiveRate"), 0.0535427, 5e-8);
}

#[test]
fn tvm_payment_timing_signs_and_zero_rate() {
    for (beginning, expected) in [(false, 12335.562372899918), (true, 12397.240184764416)] {
        let request = FinanceRequest::Tvm {
            target: TvmTarget::FutureValue,
            rate: 0.06,
            compounding: 12,
            payments: 12,
            periods: 12,
            beginning,
            present_value: 0.0,
            future_value: 0.0,
            payment: -1000.0,
        };
        close(
            number(&calculate(&request).unwrap(), "futureValue"),
            expected,
            1e-8,
        );
    }
    let microsoft = FinanceRequest::Tvm {
        target: TvmTarget::FutureValue,
        rate: 0.06,
        compounding: 12,
        payments: 12,
        periods: 10,
        beginning: true,
        present_value: -500.0,
        future_value: 0.0,
        payment: -200.0,
    };
    close(
        number(&calculate(&microsoft).unwrap(), "futureValue"),
        2581.40,
        0.005,
    );
    for rate in [0.0, 1e-14, -0.01] {
        let request = FinanceRequest::Tvm {
            target: TvmTarget::FutureValue,
            rate,
            compounding: 12,
            payments: 12,
            periods: 12,
            beginning: false,
            present_value: -5000.0,
            future_value: 0.0,
            payment: -100.0,
        };
        let fv = number(&calculate(&request).unwrap(), "futureValue");
        let inverse = FinanceRequest::Tvm {
            target: TvmTarget::PresentValue,
            rate,
            compounding: 12,
            payments: 12,
            periods: 12,
            beginning: false,
            present_value: 0.0,
            future_value: fv,
            payment: -100.0,
        };
        close(
            number(&calculate(&inverse).unwrap(), "presentValue"),
            -5000.0,
            1e-8,
        );
        if rate == 0.0 {
            close(fv, 6200.0, 1e-8);
        }
    }
    let pmt = FinanceRequest::Tvm {
        target: TvmTarget::Payment,
        rate: 0.0,
        compounding: 1,
        payments: 12,
        periods: 12,
        beginning: false,
        present_value: 12000.0,
        future_value: 0.0,
        payment: 0.0,
    };
    close(number(&calculate(&pmt).unwrap(), "payment"), -1000.0, 1e-10);
}

#[test]
fn differing_compounding_and_payment_frequencies() {
    let result = calculate(&FinanceRequest::Tvm {
        target: TvmTarget::FutureValue,
        rate: 0.12,
        compounding: 2,
        payments: 12,
        periods: 12,
        beginning: false,
        present_value: -10000.0,
        future_value: 0.0,
        payment: 0.0,
    })
    .unwrap();
    close(number(&result, "futureValue"), 11236.0, 1e-8);
}

#[test]
fn calendar_dates_ignore_timezones_and_split_leap_years() {
    let calculate_dates = |start: &str, end: &str, basis| {
        calculate(&FinanceRequest::DatedSimple {
            principal: 100.0,
            rate: 0.1,
            start: start.into(),
            end: end.into(),
            basis,
        })
    };
    close(
        number(
            &calculate_dates("2024-02-28", "2024-03-01", DayBasis::Act365F).unwrap(),
            "days",
        ),
        2.0,
        0.0,
    );
    close(
        number(
            &calculate_dates("2023-12-31", "2024-01-02", DayBasis::ActActIsda).unwrap(),
            "years",
        ),
        1.0 / 365.0 + 1.0 / 366.0,
        1e-12,
    );
    assert!(calculate_dates("2023-02-29", "2024-01-01", DayBasis::Act360).is_err());
    assert!(calculate_dates("2024-1-01", "2024-01-02", DayBasis::Act360).is_err());
    assert!(calculate_dates("2024-01-02", "2024-01-02", DayBasis::Act360).is_err());
}

#[test]
fn coupon_date_bond_matches_analytic_price_duration_and_scaling() {
    let result = calculate(&bond_request()).unwrap();
    close(number(&result, "cleanPrice"), 91.88910422064494, 1e-10);
    close(number(&result, "accruedInterest"), 0.0, 0.0);
    close(
        number(&result, "macaulayDuration"),
        4.361457867036523,
        1e-10,
    );
    close(
        number(&result, "modifiedDuration"),
        4.193709487535118,
        1e-10,
    );
    close(number(&result, "dv01"), 0.038535620817122196, 1e-12);
    close(number(&result, "holdingDv01"), 385.35620817122196, 1e-8);
    assert_eq!(result.cash_flows.len(), 10);
    assert_eq!(result.cash_flows.last().unwrap().amount, 103.0);
}

#[test]
fn official_excel_bond_examples_with_independent_inputs() {
    let mut request = bond_request();
    if let FinanceRequest::Bond {
        settlement,
        maturity,
        previous_coupon,
        next_coupon,
        basis,
        coupon,
        yield_rate,
        ..
    } = &mut request
    {
        *settlement = "2008-02-15".into();
        *maturity = "2017-11-15".into();
        *previous_coupon = "2007-11-15".into();
        *next_coupon = "2008-05-15".into();
        *basis = BondBasis::Us30_360;
        *coupon = 0.0575;
        *yield_rate = 0.065;
    }
    close(
        number(&calculate(&request).unwrap(), "cleanPrice"),
        94.63,
        0.005,
    );
    if let FinanceRequest::Bond {
        target,
        maturity,
        price,
        ..
    } = &mut request
    {
        *target = BondTarget::Yield;
        *maturity = "2016-11-15".into();
        *price = 95.04287;
    }
    close(
        number(&calculate(&request).unwrap(), "yieldRate"),
        0.065,
        1e-7,
    );
    if let FinanceRequest::Bond {
        target,
        settlement,
        maturity,
        previous_coupon,
        next_coupon,
        basis,
        coupon,
        yield_rate,
        ..
    } = &mut request
    {
        *target = BondTarget::Price;
        *settlement = "2008-01-01".into();
        *maturity = "2016-01-01".into();
        *previous_coupon = "2008-01-01".into();
        *next_coupon = "2008-07-01".into();
        *basis = BondBasis::ActActCoupon;
        *coupon = 0.08;
        *yield_rate = 0.09;
    }
    close(
        number(&calculate(&request).unwrap(), "modifiedDuration"),
        5.736,
        0.0005,
    );
}

#[test]
fn last_coupon_uses_simple_discount_and_consistent_derivative() {
    let mut request = bond_request();
    if let FinanceRequest::Bond { maturity, .. } = &mut request {
        *maturity = "2027-04-01".into();
    }
    let result = calculate(&request).unwrap();
    close(number(&result, "dirtyPrice"), 103.0 / 1.04, 1e-10);
    close(number(&result, "modifiedDuration"), 0.5 / 1.04, 1e-10);
    close(
        number(&result, "convexity"),
        2.0 * (0.5_f64 / 1.04).powi(2),
        1e-10,
    );
    if let FinanceRequest::Bond { settlement, .. } = &mut request {
        *settlement = "2027-01-01".into();
    }
    let result = calculate(&request).unwrap();
    let t = 90.0 / 182.0 / 2.0;
    close(
        number(&result, "dirtyPrice"),
        103.0 / (1.0 + 0.08 * t),
        1e-10,
    );
    close(
        number(&result, "modifiedDuration"),
        t / (1.0 + 0.08 * t),
        1e-10,
    );
}

#[test]
fn bond_inverse_and_derivative_across_coupon_dates_and_month_ends() {
    for (settle, maturity, previous, next, eom) in [
        (
            "2026-10-12",
            "2031-10-01",
            "2026-10-01",
            "2027-04-01",
            false,
        ),
        ("2024-02-28", "2026-08-31", "2023-08-31", "2024-02-29", true),
        ("2024-02-29", "2026-08-31", "2024-02-29", "2024-08-31", true),
        ("2024-03-01", "2026-08-31", "2024-02-29", "2024-08-31", true),
    ] {
        for basis in [BondBasis::ActActCoupon, BondBasis::Us30_360] {
            let mut request = bond_request();
            if let FinanceRequest::Bond {
                settlement,
                maturity: mat,
                previous_coupon,
                next_coupon,
                end_of_month,
                basis: b,
                ..
            } = &mut request
            {
                *settlement = settle.into();
                *mat = maturity.into();
                *previous_coupon = previous.into();
                *next_coupon = next.into();
                *end_of_month = eom;
                *b = basis;
            }
            let result = calculate(&request).unwrap();
            let clean = number(&result, "cleanPrice");
            close(
                clean + number(&result, "accruedInterest"),
                number(&result, "dirtyPrice"),
                1e-10,
            );
            for dirty in [false, true] {
                let mut inverse = request.clone();
                if let FinanceRequest::Bond {
                    target,
                    price,
                    dirty_input,
                    ..
                } = &mut inverse
                {
                    *target = BondTarget::Yield;
                    *price = if dirty {
                        number(&result, "dirtyPrice")
                    } else {
                        clean
                    };
                    *dirty_input = dirty;
                }
                close(
                    number(&calculate(&inverse).unwrap(), "yieldRate"),
                    0.08,
                    1e-10,
                );
            }
            let prices: Vec<f64> = [-1e-5, 1e-5]
                .into_iter()
                .map(|delta| {
                    let mut bumped = request.clone();
                    if let FinanceRequest::Bond { yield_rate, .. } = &mut bumped {
                        *yield_rate += delta;
                    }
                    number(&calculate(&bumped).unwrap(), "dirtyPrice")
                })
                .collect();
            close(
                (prices[0] - prices[1]) / 2e-5 * 0.0001,
                number(&result, "dv01"),
                1e-8,
            );
        }
    }
}

#[test]
fn rejects_unsupported_inputs_and_numeric_failures() {
    assert!(calculate(&FinanceRequest::Compound {
        principal: 10.0,
        rate: 1.0,
        periods: 100_000,
        compounding: 1
    })
    .is_err());
    assert!(calculate(&FinanceRequest::Compound {
        principal: 10.0,
        rate: -1.0,
        periods: 1,
        compounding: 1
    })
    .is_err());
    assert!(calculate(&FinanceRequest::Simple {
        principal: f64::NAN,
        rate: 0.1,
        years: 1.0
    })
    .is_err());
    let mut request = bond_request();
    if let FinanceRequest::Bond {
        previous_coupon, ..
    } = &mut request
    {
        *previous_coupon = "2026-09-30".into();
    }
    assert_eq!(calculate(&request).unwrap_err().field, "previousCoupon");
    let mut request = bond_request();
    if let FinanceRequest::Bond { target, price, .. } = &mut request {
        *target = BondTarget::Yield;
        *price = 200.0;
    }
    assert!(number(&calculate(&request).unwrap(), "yieldRate") < 0.0);
    let request: FinanceRequest = serde_json::from_str(
        r#"{"kind":"compound","principal":100,"rate":0.03,"periods":2,"compounding":1}"#,
    )
    .unwrap();
    assert!(
        serde_json::to_value(calculate(&request).unwrap()).unwrap()["engineVersion"].is_string()
    );
}

#[test]
fn automatic_coupon_schedule_and_negative_yields_with_convexity() {
    for frequency in [1, 2, 3, 4] {
        for y in [-0.01, 0.0, 0.08] {
            let mut request = bond_request();
            if let FinanceRequest::Bond {
                previous_coupon,
                next_coupon,
                frequency: f,
                yield_rate,
                ..
            } = &mut request
            {
                previous_coupon.clear();
                next_coupon.clear();
                *f = frequency;
                *yield_rate = y;
            }
            let result = calculate(&request).unwrap();
            assert_eq!(
                result.schedule.as_ref().unwrap().remaining_payments,
                5 * frequency as usize
            );
            let mut inverse = request.clone();
            if let FinanceRequest::Bond { target, price, .. } = &mut inverse {
                *target = BondTarget::Yield;
                *price = number(&result, "cleanPrice");
            }
            close(number(&calculate(&inverse).unwrap(), "yieldRate"), y, 1e-10);
            let prices: Vec<f64> = [-1e-4, 1e-4]
                .into_iter()
                .map(|delta| {
                    let mut bumped = request.clone();
                    if let FinanceRequest::Bond { yield_rate, .. } = &mut bumped {
                        *yield_rate += delta;
                    }
                    number(&calculate(&bumped).unwrap(), "dirtyPrice")
                })
                .collect();
            let p = number(&result, "dirtyPrice");
            close(
                (prices[0] + prices[1] - 2.0 * p) / 1e-8 / p,
                number(&result, "convexity"),
                1e-5,
            );
        }
    }
    // Omitted dates also work through the public JSON interface.
    let mut json = serde_json::to_value(bond_request()).unwrap();
    json.as_object_mut().unwrap().remove("previousCoupon");
    json.as_object_mut().unwrap().remove("nextCoupon");
    let request = serde_json::from_value(json).unwrap();
    assert_eq!(
        calculate(&request).unwrap().schedule.unwrap().next_coupon,
        "2027-04-01"
    );
}

fn single_request() -> FinanceRequest {
    FinanceRequest::SingleBond {
        target: BondTarget::Price,
        profile: SingleBondKind::MaturityInterest,
        settlement: "2008-02-15".into(),
        maturity: "2008-04-13".into(),
        issue: "2007-11-11".into(),
        basis: DayBasis::Us30_360,
        coupon: 0.061,
        redemption: 100.0,
        yield_rate: 0.061,
        price: 100.0,
        dirty_input: false,
        holding: 1000.0,
    }
}

#[test]
fn maturity_interest_matches_official_pricemat_and_yieldmat_examples() {
    // Microsoft PRICEMAT (basis 0), independently published rounded price.
    let result = calculate(&single_request()).unwrap();
    close(number(&result, "cleanPrice"), 99.98, 0.005);
    close(
        number(&result, "accruedInterest"),
        100.0 * 0.061 * 94.0 / 360.0,
        1e-12,
    );
    let mut request = single_request();
    if let FinanceRequest::SingleBond {
        target,
        settlement,
        maturity,
        issue,
        coupon,
        price,
        ..
    } = &mut request
    {
        *target = BondTarget::Yield;
        *settlement = "2008-03-15".into();
        *maturity = "2008-11-03".into();
        *issue = "2007-11-08".into();
        *coupon = 0.0625;
        *price = 100.0123;
    }
    // Microsoft YIELDMAT publishes 0.060954, not just the two-decimal display.
    close(
        number(&calculate(&request).unwrap(), "yieldRate"),
        0.060954,
        5e-7,
    );
}

#[test]
fn discount_and_maturity_bonds_roundtrip_and_reject_invalid_dates() {
    for profile in [SingleBondKind::Discount, SingleBondKind::MaturityInterest] {
        for basis in [
            DayBasis::Act365F,
            DayBasis::Act360,
            DayBasis::ActActIsda,
            DayBasis::Us30_360,
        ] {
            for y in [-0.02, 0.0, 0.08] {
                let mut request = single_request();
                if let FinanceRequest::SingleBond {
                    profile: p,
                    basis: b,
                    yield_rate,
                    ..
                } = &mut request
                {
                    *p = profile;
                    *b = basis;
                    *yield_rate = y;
                }
                let result = calculate(&request).unwrap();
                for dirty in [false, true] {
                    let mut inverse = request.clone();
                    if let FinanceRequest::SingleBond {
                        target,
                        price,
                        dirty_input,
                        ..
                    } = &mut inverse
                    {
                        *target = BondTarget::Yield;
                        *dirty_input = dirty;
                        *price = number(&result, if dirty { "dirtyPrice" } else { "cleanPrice" });
                    }
                    close(number(&calculate(&inverse).unwrap(), "yieldRate"), y, 1e-12);
                }
                if profile == SingleBondKind::Discount {
                    close(number(&result, "accruedInterest"), 0.0, 0.0);
                }
                close(
                    number(&result, "holdingValue"),
                    10.0 * number(&result, "dirtyPrice"),
                    1e-10,
                );
                let bumped: Vec<f64> = [-1e-4, 1e-4]
                    .into_iter()
                    .map(|delta| {
                        let mut request = request.clone();
                        if let FinanceRequest::SingleBond { yield_rate, .. } = &mut request {
                            *yield_rate += delta;
                        }
                        number(&calculate(&request).unwrap(), "dirtyPrice")
                    })
                    .collect();
                let p = number(&result, "dirtyPrice");
                close(
                    (bumped[0] + bumped[1] - 2.0 * p) / 1e-8 / p,
                    number(&result, "convexity"),
                    1e-6,
                );
            }
        }
    }
    let mut request = single_request();
    if let FinanceRequest::SingleBond { issue, .. } = &mut request {
        *issue = "2008-03-01".into();
    }
    assert_eq!(calculate(&request).unwrap_err().field, "issue");
    if let FinanceRequest::SingleBond {
        profile,
        issue,
        settlement,
        maturity,
        yield_rate,
        basis,
        ..
    } = &mut request
    {
        *profile = SingleBondKind::Discount;
        issue.clear();
        *settlement = "2026-01-01".into();
        *maturity = "2027-01-01".into();
        *basis = DayBasis::Act365F;
        *yield_rate = -1.0;
    }
    assert_eq!(calculate(&request).unwrap_err().code, "rateDomain");
}
