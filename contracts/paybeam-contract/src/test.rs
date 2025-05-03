#![cfg(test)]
extern crate std;

use crate::contract::{Token, TokenClient};
use crate::{Contract, ContractClient, Invoice};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Ledger},
    Address, Env, IntoVal, Symbol, Vec, Map
};

fn create_paybeam_contract<'a>(e: &Env) -> ContractClient<'a> {
    let contract = e.register_contract(None, Contract);
    ContractClient::new(e, &contract)
}

fn create_token<'a>(e: &Env, admin: &Address) -> TokenClient<'a> {
    let token = e.register_contract(None, Token);
    TokenClient::new(e, &token)
}

#[test]
fn test_initialize() {
    let e = Env::default();
    let contract = create_paybeam_contract(&e);
    let token_admin = Address::generate(&e);
    let token = create_token(&e, &token_admin);
    
    contract.initialize(&token.address);
    
    // Verify token address was set
    let stored_token: Address = e.storage().persistent()
        .get(&Symbol::new(&e, "token"))
        .unwrap();
    assert_eq!(stored_token, token.address);
}

#[test]
fn test_create_invoice() {
    let e = Env::default();
    e.mock_all_auths();
    e.ledger().set_timestamp(123456789);
    
    let contract = create_paybeam_contract(&e);
    let token_admin = Address::generate(&e);
    // let token = create_token(&e, &token_admin);
    // contract.initialize(&token.address);
    
    let merchant = Address::generate(&e);
    let invoice_id = Symbol::new(&e, "INV001");
    let memo = Symbol::new(&e, "MEMO123");
    
    // Create invoice
    let created_id = contract.create_invoice(
        &invoice_id,
        100,
        e.ledger().timestamp() + 3600, // 1 hour from now
        &merchant,
        &memo
    );
    
    assert_eq!(created_id, invoice_id);
    
    // Verify invoice storage
    let invoice: Invoice = e.storage().instance().get(&invoice_id).unwrap();
    assert_eq!(invoice.total_amount, 100);
    assert_eq!(invoice.merchant, merchant);
    assert!(!invoice.paid);
    
    // Test memo lookup
    let memo_invoice = contract.get_invoice_by_memo(&memo).unwrap();
    assert_eq!(memo_invoice.total_amount, 100);
}

#[test]
#[should_panic(expected = "Invoice ID already exists")]
fn test_duplicate_invoice() {
    let e = Env::default();
    let contract = create_paybeam_contract(&e);
    let token_admin = Address::generate(&e);
    let token = create_token(&e, &token_admin);
    contract.initialize(&token.address);
    
    let merchant = Address::generate(&e);
    let invoice_id = Symbol::new(&e, "INV001");
    let memo = Symbol::new(&e, "MEMO123");
    
    contract.create_invoice(
        &invoice_id,
        100,
        e.ledger().timestamp() + 3600,
        &merchant,
        &memo
    );
    
    // Should panic
    contract.create_invoice(
        &invoice_id,
        100,
        e.ledger().timestamp() + 3600,
        &merchant,
        &memo
    );
}

#[test]
fn test_pay_invoice() {
    let e = Env::default();
    e.mock_all_auths();
    e.ledger().set_timestamp(123456789);
    
    let contract = create_paybeam_contract(&e);
    let token_admin = Address::generate(&e);
    let token = create_token(&e, &token_admin);
    contract.initialize(&token.address);
    
    // Setup
    let merchant = Address::generate(&e);
    let payer = Address::generate(&e);
    let invoice_id = Symbol::new(&e, "INV001");
    let memo = Symbol::new(&e, "MEMO123");
    
    // Mint tokens to payer
    token.mint(&payer, &1000);
    
    // Create invoice
    contract.create_invoice(
        &invoice_id,
        100,
        e.ledger().timestamp() + 3600,
        &merchant,
        &memo
    );
    
    // Pay invoice
    contract.pay_invoice(&invoice_id, &payer, 100);
    
    // Verify state
    let invoice = contract.get_invoice(&invoice_id);
    assert!(invoice.paid);
    assert_eq!(invoice.payments.get(payer.clone()).unwrap(), 100);
    
    // Verify merchant received funds
    assert_eq!(token.balance(&merchant), 100);
}

#[test]
#[should_panic(expected = "Invoice expired")]
fn test_expired_invoice_payment() {
    let e = Env::default();
    e.mock_all_auths();
    e.ledger().set_timestamp(123456789);
    
    let contract = create_paybeam_contract(&e);
    let token_admin = Address::generate(&e);
    let token = create_token(&e, &token_admin);
    contract.initialize(&token.address);
    
    let merchant = Address::generate(&e);
    let payer = Address::generate(&e);
    let invoice_id = Symbol::new(&e, "INV001");
    let memo = Symbol::new(&e, "MEMO123");
    
    // Create expired invoice
    contract.create_invoice(
        &invoice_id,
        100,
        e.ledger().timestamp() - 1, // Already expired
        &merchant,
        &memo
    );
    
    // Should panic
    contract.pay_invoice(&invoice_id, &payer, 100);
}

#[test]
fn test_refund_flow() {
    let e = Env::default();
    e.mock_all_auths();
    e.ledger().set_timestamp(123456789);
    
    let contract = create_paybeam_contract(&e);
    let token_admin = Address::generate(&e);
    let token = create_token(&e, &token_admin);
    contract.initialize(&token.address);
    
    let merchant = Address::generate(&e);
    let payer = Address::generate(&e);
    let invoice_id = Symbol::new(&e, "INV001");
    let memo = Symbol::new(&e, "MEMO123");
    
    // Mint tokens to payer
    token.mint(&payer, &1000);
    
    // Create invoice
    contract.create_invoice(
        &invoice_id,
        100,
        e.ledger().timestamp() + 3600,
        &merchant,
        &memo
    );
    
    // Make partial payment
    contract.pay_invoice(&invoice_id, &payer, 50);
    
    // Expire invoice
    e.ledger().set_timestamp(e.ledger().timestamp() + 4000);
    assert!(contract.expire_invoice(&invoice_id));
    
    // Refund
    assert!(contract.refund_payment(&invoice_id, &payer));
    
    // Verify refund
    assert_eq!(token.balance(&payer), 950); // 1000 - 50 + 50 refund
}

// Mock Token contract for testing
mod token {
    use soroban_sdk::{contractimpl, Address, Env, Symbol, Val, Vec};
    
    pub struct Token;
    
    #[contractimpl]
    impl Token {
        pub fn mint(env: Env, to: Address, amount: i128) {
            to.require_auth();
            env.storage().set(&to, &amount);
        }
        
        pub fn balance(env: Env, id: Address) -> i128 {
            env.storage().get(&id).unwrap_or(0)
        }
        
        pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
            from.require_auth();
            let balance_from = Self::balance(env.clone(), from.clone());
            env.storage().set(&from, &(balance_from - amount));
            
            let balance_to = Self::balance(env.clone(), to.clone());
            env.storage().set(&to, &(balance_to + amount));
        }
    }
}