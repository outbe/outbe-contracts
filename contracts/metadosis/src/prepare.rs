use crate::error::ContractError;
use crate::state::{LysisInfo, MetadosisInfo, TouchInfo, CONFIG, METADOSIS_INFO};
use cosmwasm_std::{Addr, Decimal, DepsMut, QuerierWrapper, Uint128};
use outbe_utils::consts::{to_decimals_amount, DECIMALS};
use outbe_utils::date::WorldwideDay;
use outbe_utils::denom::{CommodityType, Currency, Denom};
use price_oracle::types::DayType;

// todo implement fees calculation
const TOTAL_FEES: Uint128 = Uint128::zero();

/// Schedules runs for the given day
pub fn prepare_executions(
    deps: DepsMut,
    total_emission_limit: Uint128,
    execution_date: WorldwideDay,
) -> Result<(), ContractError> {
    if METADOSIS_INFO.has(deps.storage, execution_date) {
        return Err(ContractError::AlreadyPrepared {
            day: execution_date,
        });
    };

    let config = CONFIG.load(deps.storage)?;
    let tribute_address = config.tribute.ok_or(ContractError::NotInitialized {})?;
    let price_oracle_address = config
        .price_oracle
        .ok_or(ContractError::NotInitialized {})?;

    let coen_usdc_rate: price_oracle::types::TokenPairPrice = deps.querier.query_wasm_smart(
        &price_oracle_address,
        &price_oracle::query::QueryMsg::GetPrice {},
    )?;

    let total_gratis_limit = (total_emission_limit - TOTAL_FEES) * DECIMALS; // NB convert to units

    let metadosis_info: MetadosisInfo = match coen_usdc_rate.day_type {
        DayType::Green => {
            let total_tribute_interest: Uint128 =
                query_total_tribute_amount(deps.querier, &tribute_address, execution_date)?;
            println!("Total tribute quantity = {}", total_tribute_interest);

            let (total_lysis_limit, total_lysis_deficit, distribution_percent) = calc_lysis_limit(
                total_gratis_limit,
                total_tribute_interest,
                config.lysis_limit_percent,
            );

            MetadosisInfo::Lysis {
                lysis_info: LysisInfo {
                    total_gratis_limit,
                    total_fees: TOTAL_FEES,
                    total_lysis_limit,
                    total_tribute_interest,
                    total_lysis_deficit,
                    distribution_percent,
                },
            }
        }
        DayType::Red => {
            let touch_limit = total_gratis_limit * Uint128::new(4) / Uint128::new(100);

            let gold_price: price_oracle::types::PriceData = deps.querier.query_wasm_smart(
                &price_oracle_address,
                &price_oracle::query::QueryMsg::GetLatestPrice {
                    token1: Denom::Commodity(CommodityType::Xau),
                    token2: Denom::Fiat(Currency::Usd),
                },
            )?;

            // NB: the bank gold ignot (400 troy ounces) price in coen
            let gold_ignot_price = gold_price.price / coen_usdc_rate.price;
            let gold_ignot_price = gold_ignot_price * Decimal::from_atomics(400u128, 0).unwrap();

            MetadosisInfo::Touch {
                touch_info: TouchInfo {
                    total_gratis_limit,
                    touch_limit,
                    gold_ignot_price,
                },
            }
        }
    };

    METADOSIS_INFO.save(deps.storage, execution_date, &metadosis_info)?;

    Ok(())
}

fn calc_lysis_limit(
    total_gratis_limit: Uint128,
    total_tribute_interest: Uint128,
    lysis_limit_percent: Decimal,
) -> (Uint128, Uint128, Decimal) {
    // take 8%
    let total_tribute_interest =
        (to_decimals_amount(total_tribute_interest) * lysis_limit_percent).atomics();

    let mut total_lysis_limit = total_gratis_limit;
    // If the interest is less than the total limit, the limit is set to the interest
    // to reduce over allocation
    if total_tribute_interest < total_lysis_limit {
        total_lysis_limit = total_tribute_interest;
        // TODO do we need to somewhere distribute the diff?
    }
    println!("Total Lysis Limit = {}", total_lysis_limit);
    let mut total_lysis_deficit = Uint128::zero();
    let mut distribution_percent = lysis_limit_percent;
    if total_tribute_interest > total_lysis_limit {
        total_lysis_deficit = total_tribute_interest - total_lysis_limit;
        distribution_percent = to_decimals_amount(total_lysis_limit) * lysis_limit_percent
            / to_decimals_amount(total_tribute_interest);
    }

    (total_lysis_limit, total_lysis_deficit, distribution_percent)
}

fn query_total_tribute_amount(
    querier: QuerierWrapper,
    addr: &Addr,
    date: WorldwideDay,
) -> Result<Uint128, ContractError> {
    let response: tribute::query::TotalInterestResponse =
        querier.query_wasm_smart(addr, &tribute::query::QueryMsg::TotalInterest { date })?;
    Ok(response.total_nominal_amount)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{Decimal, Uint128};

    #[test]
    fn test_calc_lysis_limit_when_interest_less_than_limit() {
        let total_gratis_limit = Uint128::new(1000);
        let total_tribute_interest = Uint128::new(500);
        let lysis_limit_percent = Decimal::percent(10);

        let (total_lysis_limit, total_lysis_deficit, distribution_percent) = calc_lysis_limit(
            total_gratis_limit,
            total_tribute_interest,
            lysis_limit_percent,
        );

        assert_eq!(total_lysis_limit, Uint128::new(50));
        assert_eq!(total_lysis_deficit, Uint128::zero());
        assert_eq!(distribution_percent, Decimal::percent(10));
    }

    #[test]
    fn test_calc_lysis_limit_when_interest_greater_than_limit() {
        let total_gratis_limit = Uint128::new(1000);
        let total_tribute_interest = Uint128::new(2000);
        let lysis_limit_percent = Decimal::percent(10);

        let (total_lysis_limit, total_lysis_deficit, distribution_percent) = calc_lysis_limit(
            total_gratis_limit,
            total_tribute_interest,
            lysis_limit_percent,
        );

        assert_eq!(total_lysis_limit, Uint128::new(200));
        assert_eq!(total_lysis_deficit, Uint128::new(0));
        assert_eq!(distribution_percent, Decimal::percent(10));
    }

    #[test]
    fn test_calc_lysis_limit_when_interest_equals_limit() {
        let total_gratis_limit = Uint128::new(1000);
        let total_tribute_interest = Uint128::new(1000);
        let lysis_limit_percent = Decimal::percent(10);

        let (total_lysis_limit, total_lysis_deficit, distribution_percent) = calc_lysis_limit(
            total_gratis_limit,
            total_tribute_interest,
            lysis_limit_percent,
        );

        assert_eq!(total_lysis_limit, Uint128::new(100));
        assert_eq!(total_lysis_deficit, Uint128::zero());
        assert_eq!(distribution_percent, Decimal::percent(10));
    }
}
