use near_sdk::near;
use near_sdk::{env, AccountId, NearToken, PanicOnDefault, Promise};
use std::collections::HashMap;

#[derive(Clone)]
#[near(serializers = [json, borsh])]
pub struct Option {
    total_deposit: NearToken,
    user_deposits: HashMap<AccountId, NearToken>,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Brawl {
    pub owner: AccountId,
    pub brawl_title: String,
    pub brawl_id: String,
    pub is_finalized: bool,
    pub correct_option: String,
    pub total_deposits: NearToken,
    pub options: HashMap<String, Option>,
    pub option_list: Vec<String>,
}

#[near]
impl Brawl {
    #[init]
    #[private] // only callable by the contract's account
    pub fn init(owner: AccountId, title: String, id: String, options: Vec<String>) -> Self {
        let mut this = Self {
            owner,
            brawl_title: title,
            brawl_id: id,
            is_finalized: false,
            correct_option: String::new(),
            total_deposits: NearToken::from_yoctonear(0),
            options: HashMap::new(),
            option_list: Vec::new(),
        };
        for option in options {
            let option_data = Option {
                total_deposit: NearToken::from_yoctonear(0),
                user_deposits: HashMap::new(),
            };
            this.options.insert(option.clone(), option_data);
            this.option_list.push(option.clone());
        }
        this
    }

    #[payable]
    pub fn deposit(&mut self, option: String) {
        assert!(!self.is_finalized, "Brawl is already finalized");

        let deposit_amount = env::attached_deposit();

        assert!(
            deposit_amount > NearToken::from_near(0),
            "Deposit amount must be greater than 0"
        );

        let mut option_data = self
            .options
            .get(&option)
            .expect("Option does not exist")
            .clone();

        option_data.total_deposit = option_data.total_deposit.saturating_add(deposit_amount);

        option_data
            .user_deposits
            .insert(env::predecessor_account_id(), deposit_amount);

        self.options.insert(option, option_data);

        self.total_deposits = self.total_deposits.saturating_add(deposit_amount);
    }

    pub fn update_correct_option(&mut self, option: String) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only the owner can update the correct option"
        );
        assert!(!self.is_finalized, "Brawl is already finalized");
        assert!(self.options.contains_key(&option), "Option does not exist");

        self.correct_option = option.clone();
        self.is_finalized = true;
    }

    pub fn claim(&mut self) -> NearToken {
        assert!(self.is_finalized, "Brawl is not finalized yet");

        let caller = env::predecessor_account_id();
        let correct_option = self.correct_option.clone();

        let option_data = self
            .options
            .get_mut(&correct_option)
            .expect("Correct option not found");
        let user_deposit = option_data
            .user_deposits
            .get(&caller)
            .cloned()
            .unwrap_or(NearToken::from_near(0));

        assert!(
            user_deposit > NearToken::from_near(0),
            "No winning deposit found"
        );

        let winning_total = option_data.total_deposit;
        let reward = ((user_deposit.saturating_mul(100).as_yoctonear()) / winning_total.as_yoctonear() )
            * (self.total_deposits.as_yoctonear() / 100);

        let reward = NearToken::from_yoctonear(reward);

        option_data.user_deposits.remove(&caller);

        Promise::new(caller.clone()).transfer(reward);

        reward
    }

    // View methods
    pub fn get_options(&self) -> Vec<String> {
        self.option_list.clone()
    }

    pub fn get_total_deposits(&self) -> NearToken {
        self.total_deposits
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    pub fn get_correct_option(&self) -> String {
        self.correct_option.clone()
    }

    pub fn get_brawl_id(&self) -> String {
        self.brawl_id.clone()
    }

    pub fn get_brawl_title(&self) -> String {
        self.brawl_title.clone()
    }

    pub fn get_is_finalized(&self) -> bool {
        self.is_finalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_contract() {
        let alice: AccountId = "alice.near".parse().unwrap();
        let contract = Brawl::init(
            alice.clone(),
            "Test Brawl".to_string(),
            "test_brawl".to_string(),
            vec!["Option 1".to_string(), "Option 2".to_string()],
        );

        let owner = contract.get_owner();
        assert_eq!(owner, alice);

        let total_deposits = contract.get_total_deposits();
        assert_eq!(total_deposits, NearToken::from_yoctonear(0));
    }
}
