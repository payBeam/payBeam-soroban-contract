use soroban_sdk::{Env, Symbol};

#[derive(Debug)]
pub enum EscrowError {
    InvoiceAlreadyExists,
    InvalidSplitAmounts,
    InvalidTotalAmount,
    InvoiceNotFound,
    InvoiceAlreadyPaid,
}

impl EscrowError {
    pub fn to_symbol(&self, env: &Env) -> Symbol {
        match self {
            EscrowError::InvoiceAlreadyExists => Symbol::new(env, "InvoiceAlreadyExists"),
            EscrowError::InvalidSplitAmounts => Symbol::new(env, "InvalidSplitAmounts"),
            EscrowError::InvalidTotalAmount => Symbol::new(env, "InvalidTotalAmount"),
            EscrowError::InvoiceNotFound => Symbol::new(env, "InvoiceNotFound"),
            EscrowError::InvoiceAlreadyPaid => Symbol::new(env, "InvoiceAlreadyPaid"),
        }
    }
}