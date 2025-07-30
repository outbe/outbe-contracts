use crate::deficit::{calc_lysis_deficits, calc_total_deficit};
use crate::error::ContractError;
use crate::state::{LysisInfo, MetadosisInfo, TouchInfo, CONFIG, METADOSIS_INFO};
use cosmwasm_std::{Addr, Decimal, DepsMut, QuerierWrapper, Uint128};
use outbe_utils::date::WorldwideDay;
use price_oracle::types::DayType;
use std::str::FromStr;

/*
Metadosis

Metadosis Day begins 36 hours after the end of Worldwide Day, at UTC 00.00.00.
Emission Limit for the corresponding Day of Worldwide Day calculated based on UTC time for the same day from 00:00:00 to 23:59.59.
Total Fees paid to Validators and Agents are calculated for this Day.
Total Gratis Limit is determined as: Total Gratis Limit = Emision Limit - Total Fees.
Total Gratis Limit is equally divided into 24 portions (23 Lysis and 1 Touch).
Total Lysis Limit is sum of 23 Lysis Limits.
Total Promis Limit is minimal from Total Gratis limit / Symbolic Rate and Unallocated Emission Limit.

> 3. Total Lysis Deficit is calculated as the maximum of (Total Tribute Interest - Total Lysis Limit) or 32% of Total Tribute Interest.
> 4. Total Lysis Limit is recalculated as (Total Tribute Interest) - (Total Lysis Deficit).

 */

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
    let vector_address = config.vector.ok_or(ContractError::NotInitialized {})?;

    let exchange_rate: price_oracle::types::TokenPairPrice = deps.querier.query_wasm_smart(
        &price_oracle_address,
        &price_oracle::query::QueryMsg::GetPrice {},
    )?;

    let touch_limit = total_emission_limit / Uint128::new(24);

    let mut total_lysis_limit =
        (total_emission_limit - TOTAL_FEES) * Uint128::new(23) / Uint128::new(24);

    let gold_ignot_price = query_ignot_price(exchange_rate.price);

    let metadosis_info: MetadosisInfo = match exchange_rate.day_type {
        DayType::Green => {
            // Total Tribute Interest is calculated as the sum of Symbolic Load recorded within each Tribute
            let total_tribute_interest: Uint128 =
                query_total_tribute_interest(deps.querier, &tribute_address, execution_date)?;
            println!("Total tribute interest = {}", total_tribute_interest);

            // Total Lysis Deficit is calculated as the maximum of
            // (Total Tribute Interest - Total Lysis Limit) or 32% of Total Tribute Interest.
            let total_lysis_deficit =
                calc_total_deficit(total_tribute_interest, total_lysis_limit, config.deficit);
            println!("Total deficit = {}", total_lysis_deficit);
            println!("Total Lysis Limit = {}", total_lysis_limit);

            // recalculate total_lysis_limit
            if total_tribute_interest - total_lysis_deficit < total_lysis_limit {
                total_lysis_limit = total_tribute_interest - total_lysis_deficit;
                println!("Total Lysis Limit Recalculated = {}", total_lysis_limit);
            }

            let lysis_limit = total_lysis_limit / Uint128::new(23);
            println!("Lysis Limit = {}", lysis_limit);

            let lysis_deficits: Vec<Uint128> = calc_lysis_deficits(total_lysis_deficit);
            println!("Lysis deficits = {:?}", lysis_deficits);

            let vector_info: vector::query::AllVectorsResponse = deps
                .querier
                .query_wasm_smart(&vector_address, &vector::query::QueryMsg::Vectors {})?;

            let vector_rates: Vec<Uint128> = vector_info
                .vectors
                .iter()
                .map(|it| it.vector_rate)
                .rev() // NB: reverse order because lysis starts from 23
                .collect();

            println!("Vector rates = {:?}", vector_rates);

            MetadosisInfo::LysisAndTouch {
                lysis_info: LysisInfo {
                    total_emission_limit,
                    total_fees: TOTAL_FEES,
                    total_lysis_limit,
                    lysis_limit,
                    total_tribute_interest,
                    total_deficit: total_lysis_deficit,
                    lysis_deficits,
                    vector_rates,
                },
                touch_info: TouchInfo {
                    total_emission_limit,
                    total_fees: TOTAL_FEES,
                    touch_limit,
                    gold_ignot_price,
                },
            }
        }
        DayType::Red => MetadosisInfo::Touch {
            touch_info: TouchInfo {
                total_emission_limit,
                total_fees: TOTAL_FEES,
                touch_limit,
                gold_ignot_price,
            },
        },
    };

    METADOSIS_INFO.save(deps.storage, execution_date, &metadosis_info)?;

    Ok(())
}

fn query_ignot_price(usd_coen_rate: Decimal) -> Decimal {
    let one_ignot_price = Decimal::from_str("3312.32").unwrap();
    // todo match decimals
    one_ignot_price * usd_coen_rate
}

fn query_total_tribute_interest(
    querier: QuerierWrapper,
    addr: &Addr,
    date: WorldwideDay,
) -> Result<Uint128, ContractError> {
    let response: tribute::query::TotalInterestResponse =
        querier.query_wasm_smart(addr, &tribute::query::QueryMsg::TotalInterest { date })?;
    Ok(response.total_symbolic_load)
}
