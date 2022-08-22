use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};

use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "<svg xmlns='http://www.w3.org/2000/svg' width='338' height='338' viewBox='0 0 338 338'><defs><clipPath id='a'><circle data-name='Эллипс 1' cx='144' cy='144' r='144' transform='translate(64 76)'/></clipPath></defs><g transform='translate(-40 -51)'><circle cx='169' cy='169' r='169' transform='translate(40 51)' fill='#ffe200'/><circle cx='144' cy='144' r='144' transform='translate(64 76)' fill='#fff'/><circle cx='16' cy='16' r='16' transform='translate(251 262)' fill='#fcc200'/><g data-name='Сгруппировать 1' fill='none' stroke='#cbcbcb' stroke-width='12' clip-path='url(#a)'><path data-name='Линия 1' d='M52.924 56.188v326.851'/><path data-name='Линия 2' d='M130.549 56.188v326.851'/><path data-name='Линия 3' d='M208.174 56.188v326.851'/><path data-name='Линия 4' d='M285.799 56.188v326.851'/><path data-name='Линия 5' d='M363.424 56.188v326.851'/><path data-name='Линия 6' d='M371.601 64.363H44.75'/><path data-name='Линия 7' d='M371.6 141.988H44.749'/><path data-name='Линия 8' d='M371.6 219.614H44.749'/><path data-name='Линия 9' d='M371.6 297.238H44.749'/><path data-name='Линия 10' d='M371.6 374.864H44.749'/></g><g font-size='45' font-family='Impact'><text transform='translate(237 199)'><tspan x='0' y='0'>S</tspan></text><text transform='translate(308 199)'><tspan x='0' y='0'>B</tspan></text><text transform='translate(85 276)'><tspan x='0' y='0'>T</tspan></text><text transform='translate(159 276)'><tspan x='0' y='0'>O</tspan></text><text transform='translate(237 276)'><tspan x='0' y='0'>K</tspan></text><text transform='translate(162 349)'><tspan x='0' y='0'>E</tspan></text><text transform='translate(237 349)'><tspan x='0' y='0'>N</tspan></text></g><g fill='none' stroke='#77491c' stroke-width='5'><path data-name='Линия 11' d='m143.3 135.3 23.4-33'/><path data-name='Линия 12' d='m143.3 102.3 23.4 33'/><path data-name='Линия 13' d='M166.7 101.1v35.4'/><path data-name='Линия 14' d='M143.3 101.1v35.4'/><path data-name='Линия 15' d='M168.5 135.3h-27'/><path data-name='Линия 16' d='M168.5 102.3h-27'/></g></g></svg>";

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new_default_meta(owner_id: AccountId, total_supply: U128) -> Self {
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Sokoban Near Fungible Token".to_string(),
                symbol: "SbToken".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 24,
            },
        )
    }

    #[init]
    pub fn new(
        owner_id: AccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
        };
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());
        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &owner_id,
            amount: &total_supply,
            memo: Some("Initial tokens supply is minted"),
        }
        .emit();
        this
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into(), TOTAL_SUPPLY.into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, (TOTAL_SUPPLY - transfer_amount));
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
}