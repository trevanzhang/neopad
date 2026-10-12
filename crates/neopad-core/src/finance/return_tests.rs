use super::*;

fn number(response: &FinanceResponse, key: &str) -> f64 {
    response.values.iter().find(|v| v.key == key).unwrap().value
}
fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}
fn holding() -> FinanceRequest {
    FinanceRequest::HoldingReturn {
        buy_date: "2026-01-01".into(),
        sell_date: "2027-01-01".into(),
        face: 100000.0,
        buy_price: 100.0,
        sell_price: 102.0,
        buy_clean: false,
        sell_clean: false,
        buy_accrued: 0.0,
        sell_accrued: 0.0,
        coupons: 3000.0,
        buy_fee: 100.0,
        sell_fee: 100.0,
        financing_cost: 0.0,
    }
}
fn xirr(input: &[(&str, f64)]) -> FinanceRequest {
    FinanceRequest::Xirr {
        flows: input
            .iter()
            .map(|(date, amount)| DatedAmount {
                date: (*date).into(),
                amount: *amount,
            })
            .collect(),
    }
}

#[test]
fn holding_return_decomposes_total_currency_gain_and_uses_initial_investment() {
    let response = calculate(&holding()).unwrap();
    close(number(&response, "holdingPriceGain"), 2000.0, 1e-9);
    close(number(&response, "holdingFees"), 200.0, 1e-9);
    close(number(&response, "holdingProfit"), 4800.0, 1e-9);
    close(number(&response, "holdingReturn"), 4800.0 / 100100.0, 1e-12);
    close(
        number(&response, "holdingAnnualSimple"),
        number(&response, "holdingReturn"),
        1e-12,
    );
    let mut financed = holding();
    if let FinanceRequest::HoldingReturn { financing_cost, .. } = &mut financed {
        *financing_cost = 2000.0;
    }
    close(
        number(&calculate(&financed).unwrap(), "holdingProfit"),
        2800.0,
        1e-9,
    );
}

#[test]
fn holding_clean_and_full_prices_are_equivalent_and_cash_amounts_scale_once() {
    let mut clean = holding();
    if let FinanceRequest::HoldingReturn {
        buy_clean,
        sell_clean,
        buy_accrued,
        sell_accrued,
        sell_price,
        ..
    } = &mut clean
    {
        *buy_clean = true;
        *sell_clean = true;
        *buy_accrued = 0.5;
        *sell_accrued = 1.0;
        *sell_price = 101.0;
    }
    let mut full = clean.clone();
    if let FinanceRequest::HoldingReturn {
        buy_clean,
        sell_clean,
        buy_price,
        sell_price,
        buy_accrued,
        ..
    } = &mut full
    {
        *buy_clean = false;
        *sell_clean = false;
        *buy_price = 100.5;
        *sell_price = 102.0;
        *buy_accrued = f64::NAN; // Not an input when the full quote is supplied.
    }
    let base = calculate(&clean).unwrap();
    close(
        number(&calculate(&full).unwrap(), "holdingProfit"),
        number(&base, "holdingProfit"),
        1e-9,
    );
    if let FinanceRequest::HoldingReturn {
        face,
        coupons,
        buy_fee,
        sell_fee,
        financing_cost,
        ..
    } = &mut clean
    {
        *face *= 10.0;
        *coupons *= 10.0;
        *buy_fee *= 10.0;
        *sell_fee *= 10.0;
        *financing_cost *= 10.0;
    }
    let scaled = calculate(&clean).unwrap();
    close(
        number(&scaled, "holdingProfit"),
        number(&base, "holdingProfit") * 10.0,
        1e-8,
    );
    close(
        number(&scaled, "holdingReturn"),
        number(&base, "holdingReturn"),
        1e-12,
    );
}

#[test]
fn holding_rejects_bad_dates_or_amounts_and_preserves_losses() {
    let mut request = holding();
    if let FinanceRequest::HoldingReturn {
        sell_price,
        coupons,
        ..
    } = &mut request
    {
        *sell_price = 90.0;
        *coupons = 0.0;
    }
    assert!(number(&calculate(&request).unwrap(), "holdingReturn") < 0.0);
    for (field, input) in [
        ("face", 0.0),
        ("buyFee", -1.0),
        ("coupons", -1.0),
        ("sellPrice", -1.0),
    ] {
        let mut json = serde_json::to_value(holding()).unwrap();
        json[field] = serde_json::json!(input);
        assert!(calculate(&serde_json::from_value(json).unwrap()).is_err());
    }
    for (buy, sell) in [
        ("2026-01-01", "2026-01-01"),
        ("2026-01-02", "2026-01-01"),
        ("2026-02-30", "2027-01-01"),
    ] {
        let mut json = serde_json::to_value(holding()).unwrap();
        json["buyDate"] = buy.into();
        json["sellDate"] = sell.into();
        assert!(calculate(&serde_json::from_value(json).unwrap()).is_err());
    }
}

#[test]
fn xirr_matches_official_microsoft_irregular_cash_flow_example() {
    let response = calculate(&xirr(&[
        ("2008-01-01", -10000.0),
        ("2008-03-01", 2750.0),
        ("2008-10-30", 4250.0),
        ("2009-02-15", 3250.0),
        ("2009-04-01", 2750.0),
    ]))
    .unwrap();
    // The published Excel result is rounded and uses iterative stopping tolerance.
    close(number(&response, "xirrRate"), 0.373362535, 5e-9);
    let rate = number(&response, "xirrRate");
    let direct_npv: f64 = [
        (-10000.0, 0.0),
        (2750.0, 60.0),
        (4250.0, 303.0),
        (3250.0, 411.0),
        (2750.0, 456.0),
    ]
    .iter()
    .map(|(amount, days)| amount / (1.0 + rate).powf(days / 365.0))
    .sum();
    assert!(direct_npv.abs() < 1e-7);
    assert!(number(&response, "xirrResidual").abs() < 1e-10);
    assert_eq!(response.cash_flows.len(), 0);
    assert_eq!(response.dated_cash_flows.unwrap().len(), 5);
}

#[test]
fn xirr_handles_borrowing_fees_negative_zero_rates_and_leap_years() {
    for (inflow, expected) in [(11000.0, 0.1), (9000.0, -0.1), (10000.0, 0.0)] {
        for sign in [-1.0, 1.0] {
            let response = calculate(&xirr(&[
                ("2026-01-01", -10000.0 * sign),
                ("2027-01-01", inflow * sign),
            ]))
            .unwrap();
            close(number(&response, "xirrRate"), expected, 1e-12);
        }
    }
    let borrowing = calculate(&xirr(&[
        ("2026-01-01", 10000.0),
        ("2026-01-01", -1000.0),
        ("2027-01-01", -10000.0),
    ]))
    .unwrap();
    close(
        number(&borrowing, "xirrRate"),
        10000.0 / 9000.0 - 1.0,
        1e-12,
    );
    assert_eq!(borrowing.dated_cash_flows.unwrap()[0].amount, 9000.0);
    let leap = calculate(&xirr(&[("2024-01-01", -10000.0), ("2025-01-01", 11000.0)])).unwrap();
    close(
        number(&leap, "xirrRate"),
        1.1_f64.powf(365.0 / 366.0) - 1.0,
        1e-12,
    );
}

#[test]
fn xirr_sorting_same_day_netting_and_amount_scaling_do_not_change_rate() {
    for scale in [1e-8, 1.0, 1e10] {
        let response = calculate(&xirr(&[
            ("2027-01-01", 5000.0 * scale),
            ("2026-01-01", -10000.0 * scale),
            ("2027-01-01", 6000.0 * scale),
            ("2026-06-01", 2.0 * scale),
            ("2026-06-01", -2.0 * scale),
        ]))
        .unwrap();
        close(number(&response, "xirrRate"), 0.1, 1e-12);
        close(number(&response, "cashFlowDays"), 365.0, 0.0);
        assert_eq!(response.dated_cash_flows.unwrap().len(), 2);
    }
    // Additional investment before the sign transition is also conventional.
    assert!(calculate(&xirr(&[
        ("2026-01-01", -10000.0),
        ("2026-06-01", -5000.0),
        ("2027-01-01", 16000.0)
    ]))
    .is_ok());
}

#[test]
fn xirr_rejects_nonconventional_ambiguous_or_missing_cash_flows() {
    for (input, code) in [
        (
            vec![
                ("2026-01-01", -100.0),
                ("2027-01-01", 230.0),
                ("2028-01-01", -132.0),
            ],
            "cashFlowPattern",
        ),
        (
            vec![("2026-01-01", 100.0), ("2027-01-01", 110.0)],
            "cashFlowSigns",
        ),
        (
            vec![("2026-01-01", -100.0), ("2026-01-01", 110.0)],
            "cashFlowDates",
        ),
        (
            vec![("2026-01-01", -100.0), ("2027-01-01", 0.0)],
            "cashFlowDates",
        ),
        (
            vec![("2026-01-01", -1e12), ("2027-01-01", 1e-8)],
            "xirrRange",
        ),
    ] {
        assert_eq!(calculate(&xirr(&input)).unwrap_err().code, code);
    }
    assert_eq!(
        calculate(&xirr(&[("2026-01-01", -100.0)]))
            .unwrap_err()
            .code,
        "cashFlowCount"
    );
    let too_many: Vec<_> = (0..201).map(|_| ("2026-01-01", 1.0)).collect();
    assert_eq!(
        calculate(&xirr(&too_many)).unwrap_err().code,
        "cashFlowCount"
    );
    let invalid = calculate(&xirr(&[("2026-01-01", -100.0), ("2026-02-30", 110.0)])).unwrap_err();
    assert_eq!(invalid.row, Some(2));
    assert_eq!(invalid.field, "cashFlowDate");
    assert_eq!(
        calculate(&xirr(&[
            ("2026-01-01", -100.0),
            ("2027-01-01", f64::INFINITY)
        ]))
        .unwrap_err()
        .row,
        Some(2)
    );
}

#[test]
fn xirr_long_spans_stay_finite_and_return_a_verified_solution() {
    let response = calculate(&xirr(&[("1600-01-01", -10000.0), ("9999-01-01", 20000.0)])).unwrap();
    let span = number(&response, "cashFlowDays");
    close(
        number(&response, "xirrRate"),
        2.0_f64.powf(365.0 / span) - 1.0,
        1e-12,
    );
    let wire = serde_json::to_value(response).unwrap();
    assert_eq!(wire["datedCashFlows"][0]["date"], "1600-01-01");
}
