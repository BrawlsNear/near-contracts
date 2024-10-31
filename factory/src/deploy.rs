use near_sdk::serde::Serialize;
use near_sdk::{env, log, near, AccountId, NearToken, Promise, PromiseError};

use crate::{Contract, ContractExt, NEAR_PER_STORAGE, NO_DEPOSIT, TGAS};

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
struct BrawlInitArgs {
    owner: AccountId, 
    title: String, 
    id: String, 
    options: Vec<String>
}

#[near]
impl Contract {
    #[payable]
    pub fn deploy_new_brawl(
        &mut self,
        title: String, 
        id: String, 
        options: Vec<String>
    ) -> Promise {
        // Assert the sub-account is valid
        let current_account = env::current_account_id().to_string();

        let sub_account: AccountId = format!("{id}.{current_account}").parse().unwrap();
        assert!(
            env::is_valid_account_id(sub_account.as_bytes()),
            "Invalid subAccount"
        );

        // Assert enough tokens are attached to create the account and deploy the contract
        let attached = env::attached_deposit();

        let code = self.code.clone().unwrap();
        let contract_bytes = code.len() as u128;
        let contract_storage_cost = NEAR_PER_STORAGE.saturating_mul(contract_bytes);
        let minimum_needed = contract_storage_cost.saturating_add(NearToken::from_millinear(100));

        assert!(
            attached >= minimum_needed,
            "Attach at least {minimum_needed} yⓃ"
        );

        let args = &BrawlInitArgs {
            owner: env::predecessor_account_id(),
            title,
            id,
            options,
        };

        let init_args = near_sdk::serde_json::to_vec(args).unwrap();

        let promise = Promise::new(sub_account.clone())
            .create_account()
            .transfer(attached)
            .deploy_contract(code)
            .function_call(
                "init".to_owned(),
                init_args,
                NO_DEPOSIT,
                TGAS.saturating_mul(5),
            );

        // Add callback
        promise.then(
            Self::ext(env::current_account_id()).deploy_new_brawl_callback(
                sub_account,
                env::predecessor_account_id(),
                attached,
            ),
        )
    }

    #[private]
    pub fn deploy_new_brawl_callback(
        &mut self,
        account: AccountId,
        user: AccountId,
        attached: NearToken,
        #[callback_result] create_deploy_result: Result<(), PromiseError>,
    ) -> bool {
        if let Ok(_result) = create_deploy_result {
            log!("Correctly created and deployed to {}", account);
            return true;
        };

        log!(
            "Error creating {}, returning {}yⓃ to {}",
            account,
            attached,
            user
        );
        Promise::new(user).transfer(attached);
        false
    }
}
