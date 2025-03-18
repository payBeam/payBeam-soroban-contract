#![no_std]
use soroban_sdk::{contract, contractimpl, vec, Env, String, Vec};

// mod pbtoken {
//     soroban_sdk::contractimport!(
//             file = "/Users/finisher/Documents/github/gdx-token/target/wasm32-unknown-unknown/release/paybeam_token_contract.wasm"
//     );
// }

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "Hello"), to]
    }
}

mod test;
