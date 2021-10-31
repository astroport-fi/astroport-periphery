use astroport_periphery::lockdrop::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse,
    UpdateConfigMsg, UserInfoResponse,
};
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::{mock_env, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{attr, to_binary, Addr, Coin, Timestamp, Uint128, Uint64};
use cw20_base::msg::ExecuteMsg as CW20ExecuteMsg;
use terra_multi_test::{App, BankKeeper, ContractWrapper, Executor, TerraMockQuerier};

fn mock_app() -> App {
    let api = MockApi::default();
    let env = mock_env();
    let bank = BankKeeper::new();
    let storage = MockStorage::new();
    let tmq = TerraMockQuerier::new(MockQuerier::new(&[]));

    App::new(api, env.block, bank, storage, tmq)
}

// Instantiate ASTRO Token Contract
fn instantiate_astro_token(app: &mut App, owner: Addr) -> Addr {
    let astro_token_contract = Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));

    let astro_token_code_id = app.store_code(astro_token_contract);

    let msg = cw20_base::msg::InstantiateMsg {
        name: String::from("Astro token"),
        symbol: String::from("ASTRO"),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: owner.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    let astro_token_instance = app
        .instantiate_contract(
            astro_token_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("ASTRO"),
            None,
        )
        .unwrap();
    astro_token_instance
}

// Instantiate Terraswap
fn instantiate_terraswap(app: &mut App, owner: Addr) -> Addr {
    // Terraswap Pair
    let terraswap_pair_contract = Box::new(ContractWrapper::new(
        terraswap_pair::contract::execute,
        terraswap_pair::contract::instantiate,
        terraswap_pair::contract::query,
    ));
    let terraswap_pair_code_id = app.store_code(terraswap_pair_contract);

    // Terraswap LP Token
    let terraswap_token_contract = Box::new(ContractWrapper::new(
        terraswap_token::contract::execute,
        terraswap_token::contract::instantiate,
        terraswap_token::contract::query,
    ));
    let terraswap_token_code_id = app.store_code(terraswap_token_contract);

    // Terraswap Factory Contract
    let terraswap_factory_contract = Box::new(ContractWrapper::new(
        terraswap_factory::contract::execute,
        terraswap_factory::contract::instantiate,
        terraswap_factory::contract::query,
    ));

    let terraswap_factory_code_id = app.store_code(terraswap_factory_contract);

    let msg = terraswap::factory::InstantiateMsg {
        pair_code_id: terraswap_pair_code_id,
        token_code_id: terraswap_token_code_id,
    };

    let terraswap_factory_instance = app
        .instantiate_contract(
            terraswap_factory_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("Terraswap_Factory"),
            None,
        )
        .unwrap();
    terraswap_factory_instance
}

// Instantiate Astroport
fn instantiate_astroport(app: &mut App, owner: Addr) -> Addr {
    let mut pair_configs = vec![];
    // Astroport Pair
    let astroport_pair_contract = Box::new(ContractWrapper::new(
        astroport_pair::contract::execute,
        astroport_pair::contract::instantiate,
        astroport_pair::contract::query,
    ));
    let astroport_pair_code_id = app.store_code(astroport_pair_contract);
    pair_configs.push(astroport::factory::PairConfig {
        code_id: astroport_pair_code_id,
        pair_type: astroport::factory::PairType::Xyk {},
        total_fee_bps: 5u16,
        maker_fee_bps: 3u16,
    });

    // Astroport Pair :: Stable
    let astroport_pair_stable_contract = Box::new(ContractWrapper::new(
        astroport_pair_stable::contract::execute,
        astroport_pair_stable::contract::instantiate,
        astroport_pair_stable::contract::query,
    ));
    let astroport_pair_stable_code_id = app.store_code(astroport_pair_stable_contract);
    pair_configs.push(astroport::factory::PairConfig {
        code_id: astroport_pair_stable_code_id,
        pair_type: astroport::factory::PairType::Stable {},
        total_fee_bps: 5u16,
        maker_fee_bps: 3u16,
    });

    // Astroport LP Token
    let astroport_token_contract = Box::new(ContractWrapper::new(
        astroport_token::contract::execute,
        astroport_token::contract::instantiate,
        astroport_token::contract::query,
    ));
    let astroport_token_code_id = app.store_code(astroport_token_contract);

    // Astroport Factory Contract
    let astroport_factory_contract = Box::new(ContractWrapper::new(
        astroport_factory::contract::execute,
        astroport_factory::contract::instantiate,
        astroport_factory::contract::query,
    ));

    let astroport_factory_code_id = app.store_code(astroport_factory_contract);

    let msg = astroport::factory::InstantiateMsg {
        /// Pair contract code IDs which are allowed to create pairs
        pair_configs: pair_configs,
        token_code_id: astroport_token_code_id,
        init_hook: None,
        fee_address: Some(Addr::unchecked("fee_address".to_string())),
        generator_address: Addr::unchecked("generator_address".to_string()),
        gov: Some(Addr::unchecked("gov".to_string())),
        owner: owner.clone().to_string(),
    };

    let astroport_factory_instance = app
        .instantiate_contract(
            astroport_factory_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("Astroport_Factory"),
            None,
        )
        .unwrap();
    astroport_factory_instance
}

// Instantiate Astroport's generator and vesting contracts
fn instantiate_generator_and_vesting(
    mut app: &mut App,
    owner: Addr,
    astro_token_instance: Addr,
    lp_token_instance: Addr,
) -> (Addr, Addr) {
    // Vesting
    let vesting_contract = Box::new(ContractWrapper::new(
        astroport_vesting::contract::execute,
        astroport_vesting::contract::instantiate,
        astroport_vesting::contract::query,
    ));
    let vesting_code_id = app.store_code(vesting_contract);

    let init_msg = astroport::vesting::InstantiateMsg {
        owner: owner.to_string(),
        token_addr: astro_token_instance.clone().to_string(),
    };

    let vesting_instance = app
        .instantiate_contract(
            vesting_code_id,
            owner.clone(),
            &init_msg,
            &[],
            "Vesting",
            None,
        )
        .unwrap();

    mint_some_astro(
        &mut app,
        owner.clone(),
        astro_token_instance.clone(),
        Uint128::new(900_000_000_000),
        owner.to_string(),
    );
    app.execute_contract(
        owner.clone(),
        astro_token_instance.clone(),
        &CW20ExecuteMsg::IncreaseAllowance {
            spender: vesting_instance.clone().to_string(),
            amount: Uint128::new(900_000_000_000),
            expires: None,
        },
        &[],
    )
    .unwrap();

    // Generator
    let generator_contract = Box::new(
        ContractWrapper::new(
            astroport_generator::contract::execute,
            astroport_generator::contract::instantiate,
            astroport_generator::contract::query,
        )
        .with_reply(astroport_generator::contract::reply),
    );

    let generator_code_id = app.store_code(generator_contract);

    let init_msg = astroport::generator::InstantiateMsg {
        allowed_reward_proxies: vec![],
        start_block: Uint64::from(app.block_info().height),
        astro_token: astro_token_instance.to_string(),
        tokens_per_block: Uint128::from(0u128),
        vesting_contract: vesting_instance.clone().to_string(),
    };

    let generator_instance = app
        .instantiate_contract(
            generator_code_id,
            owner.clone(),
            &init_msg,
            &[],
            "Guage",
            None,
        )
        .unwrap();

    let tokens_per_block = Uint128::new(10_000000);

    let msg = astroport::generator::ExecuteMsg::SetTokensPerBlock {
        amount: tokens_per_block,
    };
    app.execute_contract(owner.clone(), generator_instance.clone(), &msg, &[])
        .unwrap();

    let msg = astroport::generator::QueryMsg::Config {};
    let res: astroport::generator::ConfigResponse = app
        .wrap()
        .query_wasm_smart(&generator_instance, &msg)
        .unwrap();
    assert_eq!(res.tokens_per_block, tokens_per_block);

    // vesting to generator:

    let current_block = app.block_info();

    let amount = Uint128::new(630720000000);

    let msg = CW20ExecuteMsg::IncreaseAllowance {
        spender: vesting_instance.clone().to_string(),
        amount,
        expires: None,
    };

    app.execute_contract(owner.clone(), astro_token_instance.clone(), &msg, &[])
        .unwrap();

    let msg = astroport::vesting::ExecuteMsg::RegisterVestingAccounts {
        vesting_accounts: vec![astroport::vesting::VestingAccount {
            address: generator_instance.to_string(),
            schedules: vec![astroport::vesting::VestingSchedule {
                start_point: astroport::vesting::VestingSchedulePoint {
                    time: current_block.time,
                    amount,
                },
                end_point: None,
            }],
        }],
    };

    app.execute_contract(owner.clone(), vesting_instance.clone(), &msg, &[])
        .unwrap();

    let msg = astroport::generator::ExecuteMsg::Add {
        alloc_point: Uint64::from(10u64),
        reward_proxy: None,
        lp_token: lp_token_instance.clone(),
        with_update: true,
    };
    app.execute_contract(
        Addr::unchecked(owner.clone()),
        generator_instance.clone(),
        &msg,
        &[],
    )
    .unwrap();

    (generator_instance, vesting_instance)
}

// Mints some ASTRO to "to" recepient
fn mint_some_astro(
    app: &mut App,
    owner: Addr,
    astro_token_instance: Addr,
    amount: Uint128,
    to: String,
) {
    let msg = cw20::Cw20ExecuteMsg::Mint {
        recipient: to.clone(),
        amount: amount,
    };
    let res = app
        .execute_contract(owner.clone(), astro_token_instance.clone(), &msg, &[])
        .unwrap();
    assert_eq!(res.events[1].attributes[1], attr("action", "mint"));
    assert_eq!(res.events[1].attributes[2], attr("to", to));
    assert_eq!(res.events[1].attributes[3], attr("amount", amount));
}

// Instantiate AUCTION Contract
fn instantiate_auction_contract(
    app: &mut App,
    owner: Addr,
    astro_token_instance: Addr,
    airdrop_instance: Addr,
    lockdrop_instance: Addr,
    pair_instance: Addr,
    lp_token_instance: Addr,
) -> (Addr, astroport_periphery::auction::InstantiateMsg) {
    let auction_contract = Box::new(ContractWrapper::new(
        astro_auction::contract::execute,
        astro_auction::contract::instantiate,
        astro_auction::contract::query,
    ));

    let auction_code_id = app.store_code(auction_contract);

    let auction_instantiate_msg = astroport_periphery::auction::InstantiateMsg {
        owner: owner.clone().to_string(),
        astro_token_address: astro_token_instance.clone().into_string(),
        airdrop_contract_address: airdrop_instance.to_string(),
        lockdrop_contract_address: lockdrop_instance.to_string(),
        astroport_lp_pool: Some(pair_instance.to_string()),
        lp_token_address: Some(lp_token_instance.to_string()),
        generator_contract: None,
        astro_rewards: Uint256::from(1000000000000u64),
        astro_vesting_duration: 7776000u64,
        lp_tokens_vesting_duration: 7776000u64,
        init_timestamp: 1_000_00,
        deposit_window: 100_000_00,
        withdrawal_window: 5_000_00,
    };

    // Init contract
    let auction_instance = app
        .instantiate_contract(
            auction_code_id,
            owner.clone(),
            &auction_instantiate_msg,
            &[],
            "auction",
            None,
        )
        .unwrap();
    (auction_instance, auction_instantiate_msg)
}

// Instantiate LOCKDROP Contract
fn instantiate_lockdrop_contract(app: &mut App, owner: Addr) -> (Addr, InstantiateMsg) {
    let lockdrop_contract = Box::new(ContractWrapper::new(
        astroport_lockdrop::contract::execute,
        astroport_lockdrop::contract::instantiate,
        astroport_lockdrop::contract::query,
    ));

    let lockdrop_code_id = app.store_code(lockdrop_contract);

    let lockdrop_instantiate_msg = InstantiateMsg {
        owner: Some(owner.clone().to_string()),
        init_timestamp: 1_000_00,
        deposit_window: 100_000_00,
        withdrawal_window: 5_000_00,
        min_lock_duration: 1u64,
        max_lock_duration: 52u64,
        weekly_multiplier: 1u64,
        weekly_divider: 12u64,
    };

    // open claim period for successful deposit
    app.update_block(|b| {
        b.height += 17280;
        b.time = Timestamp::from_seconds(900_00)
    });

    // Init contract
    let lockdrop_instance = app
        .instantiate_contract(
            lockdrop_code_id,
            owner.clone(),
            &lockdrop_instantiate_msg,
            &[],
            "lockdrop",
            None,
        )
        .unwrap();
    (lockdrop_instance, lockdrop_instantiate_msg)
}

#[test]
fn proper_initialization_lockdrop() {
    let mut app = mock_app();
    let owner = Addr::unchecked("contract_owner");
    let (lockdrop_instance, lockdrop_instantiate_msg) =
        instantiate_lockdrop_contract(&mut app, owner);

    let resp: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&lockdrop_instance, &QueryMsg::Config {})
        .unwrap();

    // Check config
    assert_eq!(
        lockdrop_instantiate_msg.owner.unwrap().to_string(),
        resp.owner
    );
    assert_eq!(None, resp.astro_token);
    assert_eq!(None, resp.auction_contract);
    assert_eq!(None, resp.generator);
    assert_eq!(lockdrop_instantiate_msg.init_timestamp, resp.init_timestamp);
    assert_eq!(lockdrop_instantiate_msg.deposit_window, resp.deposit_window);
    assert_eq!(
        lockdrop_instantiate_msg.withdrawal_window,
        resp.withdrawal_window
    );
    assert_eq!(
        lockdrop_instantiate_msg.min_lock_duration,
        resp.min_lock_duration
    );
    assert_eq!(
        lockdrop_instantiate_msg.max_lock_duration,
        resp.max_lock_duration
    );
    assert_eq!(
        lockdrop_instantiate_msg.weekly_multiplier,
        resp.weekly_multiplier
    );
    assert_eq!(lockdrop_instantiate_msg.weekly_divider, resp.weekly_divider);
    assert_eq!(None, resp.lockdrop_incentives);

    // Check state
    let resp: StateResponse = app
        .wrap()
        .query_wasm_smart(&lockdrop_instance, &QueryMsg::State {})
        .unwrap();

    assert_eq!(0u64, resp.total_incentives_share);
    assert_eq!(Uint128::zero(), resp.total_astro_delegated);
    assert_eq!(Uint128::zero(), resp.total_astro_returned_available);
    assert_eq!(false, resp.are_claims_allowed);
}