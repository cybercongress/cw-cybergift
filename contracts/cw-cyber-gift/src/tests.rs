#[cfg(test)]
mod tests {
    use std::borrow::BorrowMut;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, from_binary, Binary, Coin, Uint128, Uint64, Empty, Addr, coins};
    use crate::msg::{AllReleaseStageStateResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, MerkleRootResponse, QueryMsg, StateResponse};
    use crate::ContractError;
    use crate::contract::{execute, instantiate, query};
    use cw_multi_test::{next_block, Contract, ContractWrapper, Executor};
    use cyber_std::{CyberMsgWrapper};
    use cw_cyber_passport::msg::ExecuteMsg as PassportExecuteMsg;
    use cyber_std_test::CyberApp;
    use csv;
    use serde::Deserialize;
    use crate::state::Config;

    const NATIVE_TOKEN: &str = "boot";
    const OWNER: &str = "owner";
    const SPACE1: &str = "space1";
    const SPACE2: &str = "space2";
    const SPACE3: &str = "space3";
    const INIT_BALANCE_OWNER: Uint128 = Uint128::new(10000000000000);
    const INIT_BALANCE_TREASURY: Uint128 = Uint128::new(570000000);
    const CF_UP: Uint128 = Uint128::new(20);
    const CF_DOWN: Uint128 = Uint128::new(10);
    const CF: Uint128 = Uint128::new(20);
    const TARGET_CLAIM: Uint64 = Uint64::new(20);

    // pub fn next_hour(block: &mut BlockInfo) {
    //     block.time = block.time.plus_seconds(3600);
    //     block.height += 1;
    // }

    pub fn contract_gift() -> Box<dyn Contract<CyberMsgWrapper, Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    pub fn contract_passport() -> Box<dyn Contract<CyberMsgWrapper, Empty>> {
        let contract = ContractWrapper::new(
            cw_cyber_passport::contract::execute,
            cw_cyber_passport::contract::instantiate,
            cw_cyber_passport::contract::query,
        )
        .with_reply(cw_cyber_passport::contract::reply);
        Box::new(contract)
    }

    pub fn contract_treasury() -> Box<dyn Contract<CyberMsgWrapper, Empty>> {
        let contract = ContractWrapper::new_with_empty(
            cw1_subkeys::contract::execute,
            cw1_subkeys::contract::instantiate,
            cw1_subkeys::contract::query,
        );
        Box::new(contract)
    }

    pub fn contract_subgraph() -> Box<dyn Contract<CyberMsgWrapper, Empty>> {
        let contract = ContractWrapper::new(
            cw_cyber_subgraph::contract::execute,
            cw_cyber_subgraph::contract::instantiate,
            cw_cyber_subgraph::contract::query,
        )
        .with_reply(cw_cyber_subgraph::contract::reply);
        Box::new(contract)
    }

    fn mock_app(init_funds: &[Coin]) -> CyberApp {
        let mut app = CyberApp::new();
        app.init_modules(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(OWNER), init_funds.to_vec())
                .unwrap();
            });
        app
    }

    fn instantiate_gift(app: &mut CyberApp, passport: String, treasury: String) -> Addr {
        let gift_id = app.store_code(contract_gift());
        let msg = crate::msg::InstantiateMsg {
            owner: Some(OWNER.to_string()),
            passport: passport.to_string(),
            treasury: treasury.to_string(),
            allowed_native: NATIVE_TOKEN.to_string(),
            initial_balance: INIT_BALANCE_TREASURY,
            coefficient_up: CF_UP,
            coefficient_down: CF_DOWN,
            coefficient: CF,
            target_claim: TARGET_CLAIM
        };
        app.instantiate_contract(gift_id, Addr::unchecked(OWNER), &msg, &[], "gift", None)
            .unwrap()
    }

    fn instantiate_passport(app: &mut CyberApp) -> Addr {
        let passport_id = app.store_code(contract_passport());
        let msg = cw_cyber_passport::msg::InstantiateMsg {
            name: "MoonPassport".to_string(),
            symbol: "MP".to_string(),
            minter: "cosmos2contract".to_string(),
            owner: OWNER.to_string(),
            name_subgraph: SPACE1.to_string(),
            avatar_subgraph: SPACE2.to_string(),
            proof_subgraph: SPACE3.to_string(),
        };
        app.instantiate_contract(passport_id, Addr::unchecked(OWNER), &msg, &[], "passport", None)
            .unwrap()
    }

    fn instantiate_treasury(app: &mut CyberApp) -> Addr {
        let treasury_id = app.store_code(contract_treasury());
        let msg = cw1_whitelist::msg::InstantiateMsg {
            admins: vec![OWNER.to_string()],
            mutable: false
        };
        app.instantiate_contract(treasury_id, Addr::unchecked(OWNER), &msg, &coins(INIT_BALANCE_TREASURY.u128(), NATIVE_TOKEN), "treasury", None)
            .unwrap()
    }

    fn instantiate_subgraph(app: &mut CyberApp, owner: String, executer: String) -> Addr {
        let treasury_id = app.store_code(contract_subgraph());
        let msg = cw_cyber_subgraph::msg::InstantiateMsg {
            admins: vec![owner.to_string()],
            executers: vec![executer.to_string()]
        };
        app.instantiate_contract(treasury_id, Addr::unchecked(OWNER), &msg, &[], "subgraph", None)
            .unwrap()
    }

    fn setup_contracts(
        app: &mut CyberApp,
    ) -> (Addr, Addr, Addr) {
        let treasury_addr = instantiate_treasury(app);
        app.update_block(next_block);

        let passport_addr = instantiate_passport(app);
        app.update_block(next_block);

        let gift_addr = instantiate_gift(app, passport_addr.to_string(), treasury_addr.to_string());
        app.update_block(next_block);

        let _res = app.execute_contract(
            Addr::unchecked(OWNER),
            treasury_addr.clone(),
            &cw1_subkeys::msg::ExecuteMsg::IncreaseAllowance::<Empty> {
                spender: gift_addr.to_string(),
                amount: Coin::new(INIT_BALANCE_TREASURY.u128(), NATIVE_TOKEN),
                expires: None,
            },
            &[],
        );
        app.update_block(next_block);

        let name_subgraph = instantiate_subgraph(app, OWNER.to_string(), passport_addr.to_string());
        let avatar_subgraph = instantiate_subgraph(app, OWNER.to_string(), passport_addr.to_string());
        let proof_subgraph = instantiate_subgraph(app, OWNER.to_string(), passport_addr.to_string());
        app.update_block(next_block);

        let _res = app.execute_contract(
            Addr::unchecked(OWNER),
            passport_addr.clone(),
            &PassportExecuteMsg::SetSubgraphs {
                name_subgraph: name_subgraph.to_string(),
                avatar_subgraph: avatar_subgraph.to_string(),
                proof_subgraph: proof_subgraph.to_string()
            },
            &[],
        );
        app.update_block(next_block);

        (gift_addr, passport_addr, treasury_addr)
    }

    #[test]
    fn proper_flow() {
        let init_funds = coins(INIT_BALANCE_OWNER.u128(), NATIVE_TOKEN);
        let mut app = mock_app(&init_funds);

        let (gift_addr, passport_addr, treasury_addr) = setup_contracts(&mut app);

        #[derive(Debug, Deserialize)]
        struct GiftData {
            amount: u128,
            nickname: String,
            avatar: String,
            ethereum_address: String,
            bostrom_address: String,
            cosmos_address: String,
            proof_ethereum_msg_sig: String,
            proof_bostrom_msg_sig: String,
            proof_cosmos_msg_sig: String,
            ethereum_proof: String,
            cosmos_proof: String,
        }

        impl GiftData {
            fn ethereum_proof_vec(&self) -> Vec<String> {
                self.ethereum_proof.replace(&['[',']','\''][..],"").split(",").map(|x| x.parse::<String>().unwrap()).collect()
            }

            fn cosmos_proof_vec(&self) -> Vec<String> {
                self.cosmos_proof.replace(&['[',']','\''][..],"").split(",").map(|x| x.parse::<String>().unwrap()).collect()
            }
        }

        let mut test_data = csv::Reader::from_path("test-data-20.csv").unwrap();
        let mut data: Vec<GiftData> = Vec::new();
        for result in test_data.deserialize() {
            let gift_data: GiftData = result.unwrap();
            data.push(gift_data)
        }

        let _res = app.execute_contract(
            Addr::unchecked(OWNER),
            gift_addr.clone(),
            &ExecuteMsg::RegisterMerkleRoot {
                merkle_root: "1d9d3cfc8527c79b1dd338d48a3ba15c4d5430bafb140a241b24364d521739cd".to_string()
            },
            &[],
        );

        for obj in &data {
            let _ = app.execute_contract(
                Addr::unchecked(obj.bostrom_address.clone()),
                passport_addr.clone(),
                &PassportExecuteMsg::CreatePassport {
                    nickname: obj.nickname.clone(),
                    avatar: obj.avatar.clone(),
                    signature: Binary::from_base64(&obj.proof_bostrom_msg_sig.clone()).unwrap(),
                },
                &[],
            );

            let _ = app.execute_contract(
                Addr::unchecked(obj.bostrom_address.clone()),
                passport_addr.clone(),
                &PassportExecuteMsg::ProofAddress {
                    nickname: obj.nickname.clone(),
                    address: obj.ethereum_address.clone(),
                    signature: Binary::from_base64(&obj.proof_ethereum_msg_sig.clone()).unwrap(),
                },
                &[],
            );

            let _ = app.execute_contract(
                Addr::unchecked(obj.bostrom_address.clone()),
                passport_addr.clone(),
                &PassportExecuteMsg::ProofAddress {
                    nickname: obj.nickname.clone(),
                    address: obj.cosmos_address.clone(),
                    signature: Binary::from_base64(&obj.proof_cosmos_msg_sig.clone()).unwrap(),
                },
                &[],
            );
        }

        fn claim_and_release(app: &mut CyberApp, gift_addr: &Addr, treasury_addr: &Addr, data: &Vec<GiftData>, a: usize, b: usize, c: usize, d: usize) {
            for i in a..b {
                let _ =app.execute_contract(
                    Addr::unchecked(data[i].bostrom_address.clone()),
                    gift_addr.clone(),
                    &ExecuteMsg::Claim {
                        nickname: data[i].nickname.clone(),
                        gift_claiming_address: data[i].cosmos_address.clone(),
                        gift_amount: Uint128::from(data[i].amount.clone()),
                        proof: data[i].cosmos_proof_vec()
                    }, &[],
                );

                let _ =app.execute_contract(
                    Addr::unchecked(data[i].bostrom_address.clone()),
                    gift_addr.clone(),
                    &ExecuteMsg::Claim {
                        nickname: data[i].nickname.clone(),
                        gift_claiming_address: data[i].ethereum_address.clone(),
                        gift_amount: Uint128::from(data[i].amount.clone()),
                        proof: data[i].ethereum_proof_vec()
                    }, &[],
                );

                let _ =app.execute_contract(
                    Addr::unchecked(data[i].bostrom_address.clone()),
                    gift_addr.clone(),
                    &ExecuteMsg::Release {
                        gift_address: data[i].cosmos_address.clone(),
                    }, &[],
                );

                let _ =app.execute_contract(
                    Addr::unchecked(data[i].bostrom_address.clone()),
                    gift_addr.clone(),
                    &ExecuteMsg::Release {
                        gift_address: data[i].ethereum_address.clone(),
                    }, &[],
                );
            }
            let rel: AllReleaseStageStateResponse = app.wrap().query_wasm_smart(gift_addr, &QueryMsg::AllReleaseStageStates {}).unwrap();
            println!("RELEASES - {:?}", rel.releases);
            println!("TREASURY BAL - {:?}", app.wrap().query_balance(treasury_addr, "boot").unwrap());

            for i in c..d {
                let _ = app.execute_contract(
                    Addr::unchecked(data[i].bostrom_address.clone()),
                    gift_addr.clone(),
                    &ExecuteMsg::Release {
                        gift_address: data[i].cosmos_address.clone(),
                    }, &[],
                );

                let _ =app.execute_contract(
                    Addr::unchecked(data[i].bostrom_address.clone()),
                    gift_addr.clone(),
                    &ExecuteMsg::Release {
                        gift_address: data[i].ethereum_address.clone(),
                    }, &[],
                );
            }
            let rel: AllReleaseStageStateResponse = app.wrap().query_wasm_smart(gift_addr, &QueryMsg::AllReleaseStageStates {}).unwrap();
            println!("RELEASES - {:?}", rel.releases);
            println!("TREASURY BAL - {:?}", app.wrap().query_balance(treasury_addr, "boot").unwrap());
            println!("RELEASES - {:?}", "-------------");
        }

        claim_and_release(app.borrow_mut(), &gift_addr, &treasury_addr, &data, 0, 5, 0, 5);
        claim_and_release(app.borrow_mut(), &gift_addr, &treasury_addr, &data, 5, 10, 0, 10);
        claim_and_release(app.borrow_mut(), &gift_addr, &treasury_addr, &data, 10, 15, 0, 15);
        let _ = app.execute_contract(
            Addr::unchecked(OWNER),
            gift_addr.clone(),
            &ExecuteMsg::UpdateCoefficients {
                new_coefficient_up: Uint128::new(10),
                new_coefficient_down: Uint128::new(5),
            }, &[],
        );
        claim_and_release(app.borrow_mut(), &gift_addr, &treasury_addr, &data, 15, 19, 0, 19);
        claim_and_release(app.borrow_mut(), &gift_addr, &treasury_addr, &data, 19, 20, 0, 20);

        let res: ConfigResponse = app.wrap().query_wasm_smart(&gift_addr, &QueryMsg::Config {}).unwrap();
        println!("Config - {:?}", res);
        let res: StateResponse = app.wrap().query_wasm_smart(&gift_addr, &QueryMsg::State {}).unwrap();
        println!("State - {:?}", res);
        let _ = app.execute_contract(
            Addr::unchecked(OWNER),
            gift_addr.clone(),
            &ExecuteMsg::UpdateCoefficients {
                new_coefficient_up: Uint128::new(5),
                new_coefficient_down: Uint128::new(2),
            }, &[],
        );
        let res: ConfigResponse = app.wrap().query_wasm_smart(&gift_addr, &QueryMsg::Config {}).unwrap();
        println!("Config - {:?}", res);
        let res: StateResponse = app.wrap().query_wasm_smart(&gift_addr, &QueryMsg::State {}).unwrap();
        println!("State - {:?}", res);

        for i in 0..20 {
            println!("PASSPORT #{:?} BAL- {:?}", i, app.wrap().query_balance(&Addr::unchecked(data[i].bostrom_address.clone()), "boot").unwrap());
        }
    }

    #[test]
    fn update_owner() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: None,
            passport: "passport".to_string(),
            target_claim: Uint64::new(4),
            allowed_native: NATIVE_TOKEN.to_string(),
            initial_balance: Uint128::new(100),
            coefficient_up: Default::default(),
            coefficient_down: Default::default(),
            coefficient: Default::default(),
            treasury: "treasury".to_string()
        };

        let env = mock_env();
        let info = mock_info(
            "owner",
            &[Coin {
                denom: NATIVE_TOKEN.to_string(),
                amount: Uint128::new(100),
            }],
        );
        let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        // update owner
        let env = mock_env();
        let info = mock_info("owner", &[]);
        let msg = ExecuteMsg::UpdateOwner {
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
        let info = mock_info("owner", &[]);
        let msg = ExecuteMsg::UpdateOwner { new_owner: None };

        let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});
    }

    #[test]
    fn register_merkle_root() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: Some("owner".to_string()),
            passport: "passport".to_string(),
            target_claim: Uint64::new(4),
            allowed_native: NATIVE_TOKEN.to_string(),
            initial_balance: Default::default(),
            coefficient_up: Default::default(),
            coefficient_down: Default::default(),
            coefficient: Default::default(),
            treasury: "treasury".to_string()
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
        let info = mock_info("owner", &[]);
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

    #[test]
    fn owner_freeze() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: Some("owner".to_string()),
            passport: "passport".to_string(),
            target_claim: Uint64::new(4),
            allowed_native: NATIVE_TOKEN.to_string(),
            initial_balance: Uint128::new(10000000000000),
            coefficient_up: Uint128::new(20),
            coefficient_down: Uint128::new(5),
            coefficient: Uint128::new(20),
            treasury: "treasury".to_string()
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

        // can register merkle root
        let env = mock_env();
        let info = mock_info("owner", &[]);
        let msg = ExecuteMsg::RegisterMerkleRoot {
            merkle_root: "5d4f48f147cb6cb742b376dce5626b2a036f69faec10cd73631c791780e150fc"
                .to_string(),
        };
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        // can update owner
        let env = mock_env();
        let info = mock_info("owner", &[]);
        let msg = ExecuteMsg::UpdateOwner {
            new_owner: Some("owner0001".to_string()),
        };

        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // freeze contract
        let env = mock_env();
        let info = mock_info("owner0001", &[]);
        let msg = ExecuteMsg::UpdateOwner { new_owner: None };

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
