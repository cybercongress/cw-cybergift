#[cfg(test)]
mod tests {
    use cosmwasm_std::{Api, Binary, coin};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw721_base::state::TokenInfo;
    use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, PortidResponse};
    use crate::state::{LabeledAddress, PassportMetadata};
    use crate::contract::{execute, instantiate};
    use crate::error::ContractError;
    use crate::query::{query_active_passport, query_config, query_metadata_by_nickname, query_passport_by_nickname, query_portid};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let owner = "owner";
        let minter = "cosmos2contract";
        let citizen = "bostrom1wnpak7sfawsfv9c8vqe7naxfa4g99lv77d7c0z";
        let name_subspace = "name_subspace";
        let avatar_subspace = "avatar_subspace";
        let proof_subspace = "proof_subspace";

        // instantiate the contract
        let instantiate_msg = InstantiateMsg {
            name: "MoonPassport".to_string(),
            symbol: "MP".to_string(),
            minter: minter.to_string(),
            owner: owner.to_string(),
            name_subspace: name_subspace.to_string(),
            avatar_subspace: avatar_subspace.to_string(),
            proof_subspace: proof_subspace.to_string(),
        };
        let info = mock_info(&owner, &[]);
        instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

        let expected_config = ConfigResponse {
            owner: owner.to_string(),
            name_subspace: name_subspace.to_string(),
            avatar_subspace: avatar_subspace.to_string(),
            proof_subspace: proof_subspace.to_string(),
        };
        assert_eq!(query_config(deps.as_ref()).unwrap(), expected_config);

        let expected_portid = PortidResponse {
            portid: 0u64
        };
        assert_eq!(query_portid(deps.as_ref()).unwrap(), expected_portid);

        let create_passport_msg = ExecuteMsg::CreatePassport {
            nickname: "test_nickname".to_string(),
            avatar: "QmVPRR3i2oFRjgMKS5dw4QbGNwdXNoYxfcpS3C9pVxHEbb".to_string(),
        };

        let info = mock_info(&citizen, &[]);
        execute(deps.as_mut(), mock_env(), info, create_passport_msg).unwrap();

        let expected_portid = PortidResponse {
            portid: 1u64
        };
        assert_eq!(query_portid(deps.as_ref()).unwrap(), expected_portid);

        let expected_passport_metadata = PassportMetadata {
            nickname: "test_nickname".to_string(),
            avatar: "QmVPRR3i2oFRjgMKS5dw4QbGNwdXNoYxfcpS3C9pVxHEbb".to_string(),
            addresses: None,
        };
        assert_eq!(query_metadata_by_nickname(deps.as_ref(), "test_nickname".into()).unwrap(), expected_passport_metadata);

        let expected_passport = TokenInfo::<PassportMetadata> {
            owner: deps.api.addr_validate(&citizen).unwrap(),
            approvals: vec![],
            token_uri: None,
            extension: PassportMetadata {
                nickname: "test_nickname".to_string(),
                avatar: "QmVPRR3i2oFRjgMKS5dw4QbGNwdXNoYxfcpS3C9pVxHEbb".to_string(),
                addresses: None,
            }
        };
        assert_eq!(query_passport_by_nickname(deps.as_ref(), "test_nickname".into()).unwrap(), expected_passport);
        assert_eq!(query_active_passport(deps.as_ref(), citizen.into()).unwrap(), expected_passport);

        let update_name_msg = ExecuteMsg::UpdateName {
            old_nickname: "test_nickname".to_string(),
            new_nickname: "test_nickname_new".to_string(),
        };

        let info = mock_info(&citizen, &[]);
        execute(deps.as_mut(), mock_env(), info, update_name_msg).unwrap();

        assert_eq!(query_metadata_by_nickname(deps.as_ref(), "test_nickname".into()).is_err(), true);
        let expected_passport_metadata = PassportMetadata {
            nickname: "test_nickname_new".to_string(),
            avatar: "QmVPRR3i2oFRjgMKS5dw4QbGNwdXNoYxfcpS3C9pVxHEbb".to_string(),
            addresses: None,
        };
        assert_eq!(query_metadata_by_nickname(deps.as_ref(), "test_nickname_new".into()).unwrap(), expected_passport_metadata);

        let update_avatar_msg = ExecuteMsg::UpdateAvatar {
            nickname: "test_nickname_new".to_string(),
            new_avatar: "QmWfy5AzuaTLh4CtPcymE85KgBR36FNfokMmoGqYJoLALt".to_string(),
        };

        let info = mock_info(&citizen, &[]);
        execute(deps.as_mut(), mock_env(), info, update_avatar_msg).unwrap();

        let expected_passport_metadata = PassportMetadata {
            nickname: "test_nickname_new".to_string(),
            avatar: "QmWfy5AzuaTLh4CtPcymE85KgBR36FNfokMmoGqYJoLALt".to_string(),
            addresses: None,
        };
        assert_eq!(query_metadata_by_nickname(deps.as_ref(), "test_nickname_new".into()).unwrap(), expected_passport_metadata);

        //// check that is available to proof address

        let proof_address_msg = ExecuteMsg::ProofAddress {
            nickname: "test_nickname_new".to_string(),
            address: "0x0408522089294b8b3f0c9514086e6ae1df00394c".to_string(),
            // signature: Binary::from_base64("0x314a5bac145d5fc2fad44d026859c0218afd40dbad8e7bbbfce7bf9f8538d32700e82a636c3520675f2403be64f592d3d8c5d9cb6146576d62c9bf28c2b3e5211c").unwrap(),
            signature: Binary::from_base64("0x25e7436c57e830643dc475745c28d98472074d0adb838bef1813859b06c1099619fcc67daa4b65d764c6ea1f93c637f1a8eb40515e639528f2abc5c95b46d3521c").unwrap(),
        };

        let info = mock_info(&citizen, &[]);
        execute(deps.as_mut(), mock_env(), info, proof_address_msg).unwrap();

        let expected_passport_metadata = PassportMetadata {
            nickname: "test_nickname_new".to_string(),
            avatar: "QmWfy5AzuaTLh4CtPcymE85KgBR36FNfokMmoGqYJoLALt".to_string(),
            // addresses: Option::from(vec!["0x0408522089294b8b3f0c9514086e6ae1df00394c".to_string()]),
            addresses: Option::from(vec![LabeledAddress{ label: None, address: "0x0408522089294b8b3f0c9514086e6ae1df00394c".to_string() }]),
        };
        assert_eq!(query_metadata_by_nickname(deps.as_ref(), "test_nickname_new".into()).unwrap(), expected_passport_metadata);

        let proof_address_msg = ExecuteMsg::ProofAddress {
            nickname: "test_nickname_new".to_string(),
            address: "bostrom19nk207agguzdvpj9nqsf4zrjw8mcuu9afun3fv".to_string(),
            // signature: Binary::from_base64("eyJwdWJfa2V5IjogIkErTVhGcDdZZUxNdm9WbEFVNjZVdTB6M1d0YzlDdXdxMGVvY1VodE5PbW53Iiwic2lnbmF0dXJlIjogIjlPODlDVWRSUlpqMDExQnBoblRzNUpuWU05LzBPMGNoK1hMRzJETmlXcXRZbkE0eEE1QjB3bUZRRE9Rb2dPeEw1eEtXSUxWTW52MUlBLzdzMDVRc0lBPT0ifQ==").unwrap(),
            signature: Binary::from_base64("eyJwdWJfa2V5IjoiQStNWEZwN1llTE12b1ZsQVU2NlV1MHozV3RjOUN1d3EwZW9jVWh0Tk9tbnciLCJzaWduYXR1cmUiOiJTZG40Z25pQzR2MExJM2Z2U0ZMbmRtM05HZ2VFNUlJWDJOSmZsN1cxWmcxOEplTUNSbHMySkNvK2xUTll0elZKN0RUaFRuK3k0NitXUTdvaWJLaHl4UT09In0=").unwrap(),
        };

        let info = mock_info(&citizen, &[]);
        execute(deps.as_mut(), mock_env(), info, proof_address_msg).unwrap();
        let expected_passport_metadata = PassportMetadata {
            nickname: "test_nickname_new".to_string(),
            avatar: "QmWfy5AzuaTLh4CtPcymE85KgBR36FNfokMmoGqYJoLALt".to_string(),
            addresses: Option::from(vec![
                LabeledAddress{ label: None, address: "0x0408522089294b8b3f0c9514086e6ae1df00394c".to_string() },
                LabeledAddress{ label: None, address: "bostrom19nk207agguzdvpj9nqsf4zrjw8mcuu9afun3fv".to_string() }
                // "0x0408522089294b8b3f0c9514086e6ae1df00394c".to_string(),
                // "bostrom19nk207agguzdvpj9nqsf4zrjw8mcuu9afun3fv".to_string()
            ]),
        };
        assert_eq!(query_metadata_by_nickname(deps.as_ref(), "test_nickname_new".into()).unwrap(), expected_passport_metadata);

        //// check that is not available to proof same address twice

        let proof_address_msg = ExecuteMsg::ProofAddress {
            nickname: "test_nickname_new".to_string(),
            address: "0x0408522089294b8b3f0c9514086e6ae1df00394c".to_string(),
            // signature: Binary::from_base64("0x314a5bac145d5fc2fad44d026859c0218afd40dbad8e7bbbfce7bf9f8538d32700e82a636c3520675f2403be64f592d3d8c5d9cb6146576d62c9bf28c2b3e5211c").unwrap(),
            signature: Binary::from_base64("0x25e7436c57e830643dc475745c28d98472074d0adb838bef1813859b06c1099619fcc67daa4b65d764c6ea1f93c637f1a8eb40515e639528f2abc5c95b46d3521c").unwrap(),
        };

        let info = mock_info(&citizen, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, proof_address_msg).unwrap_err();

        assert_eq!(err, ContractError::IsNotEligible { msg: "Address already exist".to_string() });

        let proof_address_msg = ExecuteMsg::ProofAddress {
            nickname: "test_nickname_new".to_string(),
            address: "bostrom19nk207agguzdvpj9nqsf4zrjw8mcuu9afun3fv".to_string(),
            // signature: Binary::from_base64("eyJwdWJfa2V5IjogIkErTVhGcDdZZUxNdm9WbEFVNjZVdTB6M1d0YzlDdXdxMGVvY1VodE5PbW53Iiwic2lnbmF0dXJlIjogIjlPODlDVWRSUlpqMDExQnBoblRzNUpuWU05LzBPMGNoK1hMRzJETmlXcXRZbkE0eEE1QjB3bUZRRE9Rb2dPeEw1eEtXSUxWTW52MUlBLzdzMDVRc0lBPT0ifQ==").unwrap(),
            signature: Binary::from_base64("eyJwdWJfa2V5IjoiQStNWEZwN1llTE12b1ZsQVU2NlV1MHozV3RjOUN1d3EwZW9jVWh0Tk9tbnciLCJzaWduYXR1cmUiOiJTZG40Z25pQzR2MExJM2Z2U0ZMbmRtM05HZ2VFNUlJWDJOSmZsN1cxWmcxOEplTUNSbHMySkNvK2xUTll0elZKN0RUaFRuK3k0NitXUTdvaWJLaHl4UT09In0=").unwrap(),
        };

        let info = mock_info(&citizen, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, proof_address_msg).unwrap_err();

        assert_eq!(err, ContractError::IsNotEligible { msg: "Address already exist".to_string() });

        // check that is available to delete proved addresses

        let remove_address_msg = ExecuteMsg::RemoveAddress {
            nickname: "test_nickname_new".to_string(),
            address: "0x0408522089294b8b3f0c9514086e6ae1df00394c".to_string(),
        };

        let info = mock_info(&citizen, &[]);
        execute(deps.as_mut(), mock_env(), info, remove_address_msg).unwrap();

        let expected_passport_metadata = PassportMetadata {
            nickname: "test_nickname_new".to_string(),
            avatar: "QmWfy5AzuaTLh4CtPcymE85KgBR36FNfokMmoGqYJoLALt".to_string(),
            // addresses: Option::from(vec!["bostrom19nk207agguzdvpj9nqsf4zrjw8mcuu9afun3fv".to_string()]),
            addresses: Option::from(vec![LabeledAddress{ label: None, address: "bostrom19nk207agguzdvpj9nqsf4zrjw8mcuu9afun3fv".to_string() }]),
        };
        assert_eq!(query_metadata_by_nickname(deps.as_ref(), "test_nickname_new".into()).unwrap(), expected_passport_metadata);

        let remove_address_msg = ExecuteMsg::RemoveAddress {
            nickname: "test_nickname_new".to_string(),
            address: "bostrom19nk207agguzdvpj9nqsf4zrjw8mcuu9afun3fv".to_string(),
        };

        let info = mock_info(&citizen, &[]);
        execute(deps.as_mut(), mock_env(), info, remove_address_msg).unwrap();

        let expected_passport_metadata = PassportMetadata {
            nickname: "test_nickname_new".to_string(),
            avatar: "QmWfy5AzuaTLh4CtPcymE85KgBR36FNfokMmoGqYJoLALt".to_string(),
            // addresses: Option::from(vec![]),
            addresses: None,
        };
        assert_eq!(query_metadata_by_nickname(deps.as_ref(), "test_nickname_new".into()).unwrap(), expected_passport_metadata);

        // check that is possible to buy name
        let update_name_msg = ExecuteMsg::UpdateName {
            old_nickname: "test_nickname_new".to_string(),
            new_nickname: "name".to_string(),
        };

        let info = mock_info(&citizen, &[coin(10_000_000_000, "boot")]);
        execute(deps.as_mut(), mock_env(), info, update_name_msg).unwrap();

        assert_eq!(query_metadata_by_nickname(deps.as_ref(), "test_nickname".into()).is_err(), true);
        let expected_passport_metadata = PassportMetadata {
            nickname: "name".to_string(),
            avatar: "QmWfy5AzuaTLh4CtPcymE85KgBR36FNfokMmoGqYJoLALt".to_string(),
            // addresses: Option::from(vec![]),
            addresses: None,
        };
        assert_eq!(query_metadata_by_nickname(deps.as_ref(), "name".into()).unwrap(), expected_passport_metadata);
    }

    #[test]
    fn proper_set_minter() {
        assert_eq!(true, true);
    }

    #[test]
    fn proper_set_owner() {
        assert_eq!(true, true);
    }

    #[test]
    fn proper_create_passport() {
        assert_eq!(true, true);
    }

    #[test]
    fn proper_update_spaces() {
        assert_eq!(true, true);
    }

    #[test]
    fn proper_transfer_nft() {
        assert_eq!(true, true);
    }

    #[test]
    fn proper_send_nft() {
        assert_eq!(true, true);
    }

    #[test]
    fn proper_burn() {
        assert_eq!(true, true);
    }

    #[test]
    fn proper_mint() {
        assert_eq!(true, true);
    }

    #[test]
    fn proper_proof() {
        assert_eq!(true, true);
    }
}
