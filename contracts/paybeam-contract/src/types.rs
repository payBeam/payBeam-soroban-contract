use soroban_sdk::{contracttype, Address};

#[contracttype]
pub struct Dispute {
    pub invoice_id: Symbol,
    pub reason: Symbol,
    pub resolved: bool,
}

#[contracttype]
pub struct Subscription {
    pub invoice_id: Symbol,
    pub interval: u64, // Interval in seconds (e.g., 2592000 for 30 days)
    pub next_due_date: u64,
}


#[contracttype]
pub enum Event {
    InvoiceCreated(Symbol),
    PaymentReceived(Symbol, Address, i128),
    InvoicePaid(Symbol),
}