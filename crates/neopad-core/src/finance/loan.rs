use super::{
    error, finish, finite, value, FinanceRequest, FinanceResponse, FinanceResult, LoanMethod,
    LoanRepayment, LoanTarget,
};

fn annuity_factor(rate: f64, periods: u32) -> f64 {
    if rate == 0.0 {
        f64::from(periods)
    } else {
        -(-f64::from(periods) * rate.ln_1p()).exp_m1() / rate
    }
}

fn first_payment(principal: f64, rate: f64, n: u32, method: LoanMethod) -> f64 {
    match method {
        LoanMethod::Annuity => principal / annuity_factor(rate, n),
        LoanMethod::EqualPrincipal => principal * (1.0 / f64::from(n) + rate),
    }
}

fn positive(number: f64, field: &'static str) -> FinanceResult<()> {
    finite(number, field)?;
    if number <= 0.0 {
        return Err(error("positive", field));
    }
    Ok(())
}

pub(super) fn calculate(request: &FinanceRequest) -> FinanceResult<FinanceResponse> {
    let FinanceRequest::Loan {
        target,
        method,
        principal,
        rate,
        payment,
        months,
        frequency,
    } = request
    else {
        unreachable!()
    };
    if ![1, 2, 4, 12].contains(frequency) {
        return Err(error("loanFrequency", "loanFrequency"));
    }
    if *months == 0 || *months > 1200 {
        return Err(error("loanTerm", "loanMonths"));
    }
    if months % (12 / frequency) != 0 {
        return Err(error("termFrequency", "loanMonths"));
    }
    let n = months / (12 / frequency);
    let f = f64::from(*frequency);
    if *target != LoanTarget::Principal {
        positive(*principal, "loanPrincipal")?;
    }
    if *target != LoanTarget::Payment {
        positive(*payment, "loanPayment")?;
    }
    if *target != LoanTarget::Rate {
        finite(*rate, "loanRate")?;
        if !(0.0..=10.0).contains(rate) {
            return Err(error("loanRateRange", "loanRate"));
        }
    }
    let (p, annual, residual) = match target {
        LoanTarget::Payment => (*principal, *rate, 0.0),
        LoanTarget::Principal => {
            let p = payment / first_payment(1.0, rate / f, n, *method);
            positive(p, "loanPrincipal")?;
            (p, *rate, 0.0)
        }
        LoanTarget::Rate => {
            let zero = principal / f64::from(n);
            let tolerance = payment.abs().max(zero.abs()) * 1e-12;
            if *payment < zero - tolerance {
                return Err(error("loanPaymentLow", "loanPayment"));
            }
            let annual = if (*payment - zero).abs() <= tolerance {
                0.0
            } else if *method == LoanMethod::EqualPrincipal {
                (payment / principal - 1.0 / f64::from(n)) * f
            } else {
                if first_payment(*principal, 10.0 / f, n, *method) < *payment {
                    return Err(error("loanRateRange", "loanPayment"));
                }
                let mut low = 0.0;
                let mut high = 10.0;
                for _ in 0..160 {
                    let mid = (low + high) * 0.5;
                    let estimate = first_payment(*principal, mid / f, n, *method);
                    if estimate > *payment {
                        high = mid;
                    } else {
                        low = mid;
                    }
                }
                (low + high) * 0.5
            };
            if !(0.0..=10.0).contains(&annual) {
                return Err(error("loanRateRange", "loanPayment"));
            }
            let residual = first_payment(*principal, annual / f, n, *method) - payment;
            if residual.abs() > tolerance {
                return Err(error("noConvergence", "loanRate"));
            }
            (*principal, annual, residual)
        }
    };
    let i = annual / f;
    let factor = annuity_factor(i, n);
    let mut opening = p;
    let mut rows = Vec::with_capacity(n as usize);
    let mut total_interest = 0.0;
    for period in 1..=n {
        // Remaining PV avoids the unstable balance*(1+i)-payment recurrence.
        let balance = match method {
            LoanMethod::Annuity => p * annuity_factor(i, n - period) / factor,
            LoanMethod::EqualPrincipal => p * f64::from(n - period) / f64::from(n),
        }
        .min(opening)
        .max(0.0);
        let principal_paid = opening - balance;
        let interest = opening * i;
        total_interest += interest;
        rows.push(LoanRepayment {
            period,
            payment: principal_paid + interest,
            principal: principal_paid,
            interest,
            balance,
        });
        opening = balance;
    }
    let mut response = finish(
        vec![
            value("loanPrincipal", p, "amount"),
            value("loanRate", annual, "percent"),
            value("firstPayment", rows[0].payment, "amount"),
            value("lastPayment", rows.last().unwrap().payment, "amount"),
            value(
                "paymentDecrease",
                if *method == LoanMethod::EqualPrincipal {
                    p / f64::from(n) * i
                } else {
                    0.0
                },
                "amount",
            ),
            value("totalInterest", total_interest, "amount"),
            value("totalRepayment", p + total_interest, "amount"),
            value("repaymentCount", f64::from(n), "periods"),
            value("loanPeriodicRate", i, "percent"),
            value("loanResidual", residual, "amount"),
        ],
        vec![],
        if *method == LoanMethod::Annuity {
            "i=r/f; n=months×f/12; A=(1−(1+i)^−n)/i (A=n if i=0); payment=P/A; remainingBalance=P×A(n−k)/A(n)"
        } else {
            "i=r/f; n=months×f/12; principalPerPeriod=P/n; payment(k)=P/n+(P−(k−1)×P/n)×i; totalInterest=P×i×(n+1)/2"
        },
    )?;
    if rows.iter().any(|r| {
        [r.payment, r.principal, r.interest, r.balance]
            .iter()
            .any(|v| !v.is_finite())
    }) {
        return Err(error("overflow", "loanPrincipal"));
    }
    response.repayments = Some(rows);
    Ok(response)
}
