use soroban_sdk::{Env, Symbol, Address, Map, Vec};
use crate::types::{InvoiceData, EscrowError};

pub struct Invoice;

impl Invoice {
    pub fn create(
        env: Env,
        invoice_id: Symbol,
        total_amount: i128,
        due_date: u64,
        recipients: Vec<Address>,
        amounts: Vec<i128>,
    ) -> Result<Symbol, EscrowError> {
        if env.storage().instance().has(&invoice_id) {
            return Err(EscrowError::InvoiceAlreadyExists);
        }

        if recipients.len() != amounts.len() {
            return Err(EscrowError::InvalidSplitAmounts);
        }

        let total_split_amount: i128 = amounts.iter().sum();
        if total_split_amount != total_amount {
            return Err(EscrowError::InvalidTotalAmount);
        }

        let invoice = InvoiceData {
            total_amount,
            due_date,
            recipients,
            amounts,
            paid: false,
            payments: Map::new(&env),
        };
        env.storage().instance().set(&invoice_id, &invoice);
        Ok(invoice_id)
    }
}