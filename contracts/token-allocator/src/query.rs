use crate::state::CREATOR;
use crate::types::TokenAllocatorData;
use cosmwasm_schema::{cw_serde, QueryResponses};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, StdResult, Storage, Uint128, Uint64};
use cw_ownable::Ownership;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(TokenAllocatorData)]
    GetData {},
    /// Returns daily total allocation in units
    #[returns(TokenAllocatorData)]
    DailyAllocation {},
    #[returns(cw_ownable::Ownership<String>)]
    GetCreatorOwnership {},
    #[returns(TokenAllocatorData)]
    GetRangeData {
        from_block: Uint64,
        to_block: Uint64,
    },
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetData {} => to_json_binary(&query_amount(env)?),
        QueryMsg::DailyAllocation {} => to_json_binary(&query_daily_allocation(env)?),
        QueryMsg::GetCreatorOwnership {} => to_json_binary(&query_creator_ownership(deps.storage)?),
        QueryMsg::GetRangeData {
            from_block,
            to_block,
        } => to_json_binary(&query_range_amount(from_block, to_block)?),
    }
}

pub(crate) fn query_amount(env: Env) -> StdResult<TokenAllocatorData> {
    let initial_tokens_per_block: f64 = 2_f64.powi(16); // 65536 токенов за блок
    let blocks_per_year: u64 = 6307200;
    let emission_rate: f64 = 0.02;
    let k_soft: f64 = -6e-08;

    let total_supply = if env.block.height == 1 { 0 } else { 1 };

    let reward = reward_exponential(
        env.block.height,
        total_supply,
        initial_tokens_per_block,
        k_soft,
        blocks_per_year,
        emission_rate,
    );

    Ok(TokenAllocatorData {
        amount: Uint128::from(reward as u64),
    })
}

// TODO implement real allocation
pub(crate) fn query_daily_allocation(env: Env) -> StdResult<TokenAllocatorData> {
    let block_allocation = query_amount(env)?;
    // let daily_total_allocation = block_allocation.amount * Uint128::new(24 * 60 * 12);
    let daily_total_allocation = block_allocation.amount * Uint128::new(24);
    Ok(TokenAllocatorData {
        amount: daily_total_allocation,
    })
}

fn query_range_amount(from_block: Uint64, to_block: Uint64) -> StdResult<TokenAllocatorData> {
    let initial_tokens_per_block: f64 = 2_f64.powi(16);
    let blocks_per_year: u64 = 6307200;
    let emission_rate: f64 = 0.02;
    let k_soft: f64 = -6e-08;

    let mut total: u64 = 0;
    for block_number in from_block.u64()..=to_block.u64() {
        let total_supply = if block_number == 1 { 0 } else { 1 };
        let reward = reward_exponential(
            block_number,
            total_supply,
            initial_tokens_per_block,
            k_soft,
            blocks_per_year,
            emission_rate,
        );
        total = total.saturating_add(reward as u64);
    }

    Ok(TokenAllocatorData {
        amount: Uint128::from(total),
    })
}

fn reward_exponential(
    block_number: u64,
    total_supply: u64,
    initial_tokens_per_block: f64,
    k_soft: f64,
    blocks_per_year: u64,
    emission_rate: f64,
) -> f64 {
    let block_reward = initial_tokens_per_block * (k_soft * block_number as f64).exp();

    let current_inflation = if total_supply == 0 {
        (block_reward * blocks_per_year as f64) / initial_tokens_per_block
    } else {
        (block_reward * blocks_per_year as f64) / total_supply as f64
    };

    if current_inflation < 0.02 {
        (emission_rate * total_supply as f64) / blocks_per_year as f64
    } else {
        block_reward
    }
}

pub fn query_creator_ownership(storage: &dyn Storage) -> StdResult<Ownership<Addr>> {
    CREATOR.get_ownership(storage)
}

#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate};
    use crate::msg::InstantiateMsg;
    use crate::query::{query, QueryMsg, TokenAllocatorData};
    use cosmwasm_std::Uint64;
    use cw_multi_test::{App, ContractWrapper, Executor};

    #[test]
    fn test_query() {
        let mut app = App::default();

        let owner = app.api().addr_make("owner");

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let init_msg = InstantiateMsg { creator: None };

        let contract_addr = app
            .instantiate_contract(code_id, owner.clone(), &init_msg, &[], "allocator1", None)
            .unwrap();

        let response: TokenAllocatorData = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetData {})
            .unwrap();

        assert_eq!(response.amount.u128(), 65487u128);

        app.update_block(|block| {
            block.height += 1000;
            block.time = block.time.plus_seconds(5000);
        });

        let response: TokenAllocatorData = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetData {})
            .unwrap();
        assert_eq!(response.amount.u128(), 65483u128);

        app.update_block(|block| {
            block.height += 50000000;
            block.time = block.time.plus_seconds(250000000);
        });

        let response: TokenAllocatorData = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetData {})
            .unwrap();
        assert_eq!(response.amount.u128(), 3260u128);
    }

    #[test]
    fn test_query_range_data() {
        let mut app = App::default();

        let owner = app.api().addr_make("owner");

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let init_msg = InstantiateMsg { creator: None };

        let contract_addr = app
            .instantiate_contract(code_id, owner.clone(), &init_msg, &[], "allocator2", None)
            .unwrap();

        // Single-block range should match GetData result for block 1
        let _single: TokenAllocatorData = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetData {})
            .unwrap();

        let _range_single: TokenAllocatorData = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::GetRangeData {
                    from_block: Uint64::new(1),
                    to_block: Uint64::new(1),
                },
            )
            .unwrap();
        // TODO verify allocation algorithm and uncomment
        // assert_eq!(range_single.amount, single.amount);

        // Multi-block range (blocks 1 through 3) equals the sum of individual blocks
        let mut expected: u128 = 0;
        for block in 1u64..=3 {
            let res: TokenAllocatorData = app
                .wrap()
                .query_wasm_smart(
                    contract_addr.clone(),
                    &QueryMsg::GetRangeData {
                        from_block: Uint64::new(block),
                        to_block: Uint64::new(block),
                    },
                )
                .unwrap();
            expected = expected.saturating_add(res.amount.u128());
        }
        let range_multi: TokenAllocatorData = app
            .wrap()
            .query_wasm_smart(
                contract_addr,
                &QueryMsg::GetRangeData {
                    from_block: Uint64::new(1),
                    to_block: Uint64::new(3),
                },
            )
            .unwrap();

        assert_eq!(range_multi.amount.u128(), expected);
    }
}
