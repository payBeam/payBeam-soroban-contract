use soroban_sdk::{Env, Symbol, Address, Map, Vec, contracttype};

#[contracttype]
pub enum Event {
    InvoiceCreated(Symbol),
    PaymentReceived(Symbol, Address, i128),
    InvoicePaid(Symbol),
}

#[contracttype]
pub struct InvoiceData {
    pub total_amount: i128,
    pub due_date: u64,
    pub recipients: Vec<Address>,
    pub amounts: Vec<i128>,
    pub paid: bool,
    pub payments: Map<Address, i128>,
}

#[contracttype]
pub struct SubscriptionData {
    pub invoice_id: Symbol,
    pub interval: u64,
    pub next_due_date: u64,
}

#[contracttype]
pub struct DisputeData {
    pub invoice_id: Symbol,
    pub reason: Symbol,
    pub resolved: bool,
}