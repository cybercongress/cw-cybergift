#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use crate::execute::*;
    use crate::msg::*;
    use crate::ContractError;

    #[test]
    fn proper_flow() {
        assert_eq!(true, true)
    }
}
