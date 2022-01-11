#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, from_binary, from_slice, BankMsg, Coin, CosmosMsg, SubMsg, Uint128, Binary};
    use serde::Deserialize;

    use crate::execute::*;
    use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, IsClaimedResponse, MerkleRootResponse, QueryMsg, ClaimMsg, ClaimerType};
    use crate::ContractError;
    use std::ops::Mul;

    const NATIVE_TOKEN: &str = "boot";

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: Some("owner0000".to_string()),
            allowed_native: NATIVE_TOKEN.to_string(),
            initial_balance: Uint128::new(10000000000000),
            coefficient_up: Uint128::new(20),
            coefficient_down: Uint128::new(5),
            coefficient: Uint128::new(20),
        };

        let env = mock_env();
        let info = mock_info(
            "addr0000",
            &[Coin {
                denom: NATIVE_TOKEN.to_string(),
                amount: Uint128::new(10000000000000),
            }]);

        // we can just call .unwrap() to assert this was a success
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        // it worked, let's query the state
        let res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!("owner0000", config.owner.unwrap().as_str());
        assert_eq!("boot", config.allowed_native.as_str());
    }

    #[test]
    fn update_config() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: None,
            allowed_native: NATIVE_TOKEN.to_string(),
            initial_balance: Uint128::new(100),
            coefficient_up: Default::default(),
            coefficient_down: Default::default(),
            coefficient: Default::default(),
        };

        let env = mock_env();
        let info = mock_info(
            "owner0000",
            &[Coin {
                denom: NATIVE_TOKEN.to_string(),
                amount: Uint128::new(100),
            }],
        );
        let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        // update owner
        let env = mock_env();
        let info = mock_info("owner0000", &[]);
        let msg = ExecuteMsg::UpdateConfig {
            new_owner: Some("owner0001".to_string()),
        };

        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!("owner0001", config.owner.unwrap().as_str());

        // Unauthorized err
        let env = mock_env();
        let info = mock_info("owner0000", &[]);
        let msg = ExecuteMsg::UpdateConfig { new_owner: None };

        let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});
    }

    #[test]
    fn register_merkle_root() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: Some("owner0000".to_string()),
            allowed_native: NATIVE_TOKEN.to_string(),
            initial_balance: Default::default(),
            coefficient_up: Default::default(),
            coefficient_down: Default::default(),
            coefficient: Default::default(),
        };

        let env = mock_env();
        let info = mock_info(
            "addr0000",
            &[Coin {
                denom: NATIVE_TOKEN.to_string(),
                amount: Uint128::new(100),
            }],
        );
        let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        // register new merkle root
        let env = mock_env();
        let info = mock_info("owner0000", &[]);
        let msg = ExecuteMsg::RegisterMerkleRoot {
            merkle_root: "634de21cde1044f41d90373733b0f0fb1c1c71f9652b905cdf159e73c4cf0d37"
                .to_string(),
        };

        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "register_merkle_root"),
                attr(
                    "merkle_root",
                    "634de21cde1044f41d90373733b0f0fb1c1c71f9652b905cdf159e73c4cf0d37"
                )
            ]
        );

        let res = query(deps.as_ref(), env, QueryMsg::MerkleRoot {}).unwrap();
        let merkle_root: MerkleRootResponse = from_binary(&res).unwrap();
        assert_eq!(
            "634de21cde1044f41d90373733b0f0fb1c1c71f9652b905cdf159e73c4cf0d37".to_string(),
            merkle_root.merkle_root
        );
    }

    const ETH_TEST: &[u8] = include_bytes!("../testdata/airdrop_stage_1_test_data_ethereum_address.json");
    const COSMOS_TEST: &[u8] = include_bytes!("../testdata/airdrop_stage_1_test_data_cosmos_address.json");

    #[derive(Deserialize, Debug)]
    struct Encoded {
        claim_msg: Binary,
        signature: Binary,
        amount: Uint128,
        root: String,
        proofs: Vec<String>
    }

    #[test]
    fn claim() {
        // Run test 1
        let mut deps = mock_dependencies();
        let eth_test_data: Encoded = from_slice(ETH_TEST).unwrap();
        let eth_test_data2: Encoded = from_slice(ETH_TEST).unwrap();

        let msg = InstantiateMsg {
            owner: Some("owner0000".to_string()),
            allowed_native: NATIVE_TOKEN.to_string(),
            initial_balance: Uint128::new(10000000000000),
            coefficient_up: Uint128::new(20),
            coefficient_down: Uint128::new(5),
            coefficient: Uint128::new(20),
        };

        let env = mock_env();
        let info = mock_info(
            "addr0000",
            &[Coin {
                denom: NATIVE_TOKEN.to_string(),
                amount: Uint128::new(10000000000000),
            }],
        );
        let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        let env = mock_env();
        let info = mock_info("owner0000", &[]);
        let msg = ExecuteMsg::RegisterMerkleRoot {
            merkle_root: eth_test_data.root,
        };
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        let claim_msg = from_binary(&eth_test_data.claim_msg).unwrap();
        let msg = ExecuteMsg::Claim {
            claim_msg,
            signature: eth_test_data.signature,
            proof: eth_test_data.proofs,
            claim_amount: eth_test_data.amount
        };

        let env = mock_env();
        let info = mock_info("addr0001", &[]);
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        let claim_msg2:ClaimMsg = from_binary(&eth_test_data2.claim_msg).unwrap();
        let expected = SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: claim_msg2.clone().target_address,
            amount: vec![Coin {
                denom: NATIVE_TOKEN.to_string(),
                amount: Uint128::new(u128::from(eth_test_data2.amount)*20),
            }],
        }));
        assert_eq!(res.messages, vec![expected]);

        assert_eq!(
            res.attributes,
            vec![
                attr("action", "claim"),
                attr("original", claim_msg2.clone().gift_claiming_address),
                attr("type", ClaimerType::Ethereum.to_string()),
                attr("target", claim_msg2.clone().target_address),
                attr("amount", eth_test_data.amount.u128().mul(20).to_string())
            ]
        );

        assert!(
            from_binary::<IsClaimedResponse>(
                &query(
                    deps.as_ref(),
                    env.clone(),
                    QueryMsg::IsClaimed {
                        address: claim_msg2.target_address
                    }
                )
                .unwrap()
            )
            .unwrap()
            .is_claimed
        );

        // Second test

        let cosmos_test_data: Encoded = from_slice(COSMOS_TEST).unwrap();
        let cosmos_test_data2: Encoded = from_slice(COSMOS_TEST).unwrap();
        // check claimed
        let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(res, ContractError::Claimed {});

        // register new drop
        let env = mock_env();
        let info = mock_info("owner0000", &[]);
        let msg = ExecuteMsg::RegisterMerkleRoot {
            merkle_root: cosmos_test_data.root,
        };
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        let claim_msg = from_binary(&cosmos_test_data.claim_msg).unwrap();
        let msg = ExecuteMsg::Claim {
            claim_msg,
            signature: cosmos_test_data.signature,
            proof: cosmos_test_data.proofs,
            claim_amount: cosmos_test_data.amount
        };

        let env = mock_env();
        let info = mock_info("addr0002", &[]);
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        let claim_msg2:ClaimMsg = from_binary(&cosmos_test_data2.claim_msg).unwrap();

        let expected = SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: claim_msg2.clone().target_address,
            amount: vec![Coin {
                denom: NATIVE_TOKEN.to_string(),
                amount: Uint128::new(u128::from(cosmos_test_data2.amount).mul(20)),
            }],
        }));
        assert_eq!(res.messages, vec![expected]);

        assert_eq!(
            res.attributes,
            vec![
                attr("action", "claim"),
                attr("original", claim_msg2.clone().gift_claiming_address),
                attr("type", ClaimerType::Cosmos.to_string()),
                attr("target", claim_msg2.clone().target_address),
                attr("amount", cosmos_test_data2.amount.u128().mul(20).to_string())
            ]
        );
    }

    #[test]
    fn owner_freeze() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: Some("owner0000".to_string()),
            allowed_native: NATIVE_TOKEN.to_string(),
            initial_balance: Uint128::new(10000000000000),
            coefficient_up: Uint128::new(20),
            coefficient_down: Uint128::new(5),
            coefficient: Uint128::new(20),
        };

        let env = mock_env();
        let info = mock_info(
            "addr0000",
            &[Coin {
            denom: NATIVE_TOKEN.to_string(),
            amount: Uint128::new(10000000000000),
        }]);
        let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        // can register merkle root
        let env = mock_env();
        let info = mock_info("owner0000", &[]);
        let msg = ExecuteMsg::RegisterMerkleRoot {
            merkle_root: "5d4f48f147cb6cb742b376dce5626b2a036f69faec10cd73631c791780e150fc"
                .to_string(),
        };
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        // can update owner
        let env = mock_env();
        let info = mock_info("owner0000", &[]);
        let msg = ExecuteMsg::UpdateConfig {
            new_owner: Some("owner0001".to_string()),
        };

        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // freeze contract
        let env = mock_env();
        let info = mock_info("owner0001", &[]);
        let msg = ExecuteMsg::UpdateConfig { new_owner: None };

        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // cannot register new drop
        let env = mock_env();
        let info = mock_info("owner0001", &[]);
        let msg = ExecuteMsg::RegisterMerkleRoot {
            merkle_root: "ebaa83c7eaf7467c378d2f37b5e46752d904d2d17acd380b24b02e3b398b3e5a"
                .to_string(),
        };
        let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});

        // cannot update config
        let env = mock_env();
        let info = mock_info("owner0001", &[]);
        let msg = ExecuteMsg::RegisterMerkleRoot {
            merkle_root: "ebaa83c7eaf7467c378d2f37b5e46752d904d2d17acd380b24b02e3b398b3e5a"
                .to_string(),
        };
        let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});
    }
}
