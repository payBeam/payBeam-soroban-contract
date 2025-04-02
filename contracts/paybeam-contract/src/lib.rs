#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, Symbol, Vec};
use soroban_sdk::token::TokenClient;
// use soroban_token_sdk::metadata::TokenMetadata;

// mod invoice;

// use invoice::Invoice;
mod pbtoken {
    soroban_sdk::contractimport!(
            file = "/Users/finisher/Documents/github/stellar/paybeam-soroban-token/target/wasm32-unknown-unknown/release/paybeam_token.wasm"
    );
}

// TODO : Use as an escrow contract
// * The contract will hold the funds until the invoice is fully paid
// * The funds will be released to the merchants once the invoice is fully paid
// * The contract will also handle refunds in case the invoice expires

// Todo : Auto staking of the funds in the contract
// * The funds in the contract will be auto-staked to earn interest

// Todo : Add a merchant address to the invoice
// * release the funds to the merchant once the invoice is fully paid



#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn create_invoice(
        env: Env,
        invoice_id: Symbol, // * Unique invoice ID
        total_amount: i128,
        due_date: u64,
        merchant: Address,
        recipients: Vec<Address>,
        amounts: Vec<i128>,
    ) -> Symbol {
        // * To ensure the invoice ID is unique
        if env.storage().instance().has(&invoice_id) {
            panic!("Invoice ID already exists");
        }

        // * To validate the split payment amounts
        if recipients.len() != amounts.len() {
            panic!("Recipients and amounts must have the same length");
        }

        let total_split_amount: i128 = amounts.iter().sum();
        if total_split_amount != total_amount {
            panic!("Total split amount must equal the invoice amount");
        }

        // * invoice details
        let invoice = Invoice {
            total_amount,
            due_date,
            recipients,
            merchant,
            amounts,
            paid: false,
            payments: Map::new(&env),
        };
        // * Save the invoice
        env.storage().instance().set(&invoice_id, &invoice);

        // * Return the id of the created invoice
        invoice_id
    }

    // Pay a portion of the invoice
    pub fn pay_invoice(
        env: Env,
        invoice_id: Symbol,
        payer: Address,
        amount: i128,
    ) {
        // * Fetch the invoice
        let mut invoice: Invoice = env.storage().instance().get(&invoice_id).unwrap_or_else(|| panic!("Invoice not found"));

        // * Check if the invoice is already paid
        if invoice.paid {
            // return false;
            panic!("Invoice is already paid");
        }

        // let addr : Address = Address::from_str("USDC_CONTRACT_ADDRESS").unwrap();

        // ! Transfer USDC from payer to the escrow contract
        let usdc_token = TokenClient::new(&env, &Address::from_str(&env, "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA")); // USDC testnet contract address
        usdc_token.transfer(&payer, &env.current_contract_address(), &amount);

        // Update the payment tracker
        let total_paid = invoice.payments.get(payer.clone()).unwrap_or(0) + amount;
        invoice.payments.set(payer, total_paid);

        // Check if the invoice is fully paid
        let total_payments: i128 = invoice.payments.values().into_iter().sum();
        if total_payments >= invoice.total_amount {
            invoice.paid = true;
            Self::release_funds(env.clone(), invoice_id.clone());
        }

        // Save the updated invoice
        env.storage().instance().set(&invoice_id, &invoice);
    }

    // * Release funds to recipients once the invoice is fully paid
    fn release_funds(env: Env, invoice_id: Symbol) {
        let invoice: Invoice = env.storage().instance().get(&invoice_id).unwrap_or_else(|| panic!("Invoice not found"));

        // Ensure the invoice is fully paid
        if !invoice.paid {
            panic!("Invoice is not fully paid");
        }

        // ! Transfer funds to each recipient
        let usdc_token = TokenClient::new(&env, &Address::from_str(&env, "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"));
        for (recipient, amount) in invoice.recipients.iter().zip(invoice.amounts.iter()) {
            usdc_token.transfer(&env.current_contract_address(), &recipient, &amount);
        }
    }

    

    pub fn expire_invoice(env: Env, invoice_id: Symbol) -> bool {
        let mut invoice: Invoice = env.storage().instance().get(&invoice_id).unwrap_or_else(|| panic!("Invoice not found"));
    
        // Check if the invoice is already paid or expired
        if invoice.paid || env.ledger().timestamp() > invoice.due_date {
            return false;
        }
    
        // Mark the invoice as expired
        invoice.paid = true; // Alternatively, add an `expired` field to the Invoice struct
        env.storage().instance().set(&invoice_id, &invoice);
        true
    }

    // * Refund a payment
    pub fn refund_payment(env: Env, invoice_id: Symbol, payer: Address) -> bool {
        let mut invoice: Invoice = env.storage().instance().get(&invoice_id).unwrap_or_else(|| panic!("Invoice not found"));
    
        // Ensure the invoice is expired
        if !invoice.paid || env.ledger().timestamp() <= invoice.due_date {
            return false;
        }
    
        // Refund the payer's contribution
        let amount = invoice.payments.get(payer.clone()).unwrap_or(0);
        if amount > 0 {
            let usdc_token = TokenClient::new(&env, &Address::from_str(&env, "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"));
            usdc_token.transfer(&env.current_contract_address(), &payer, &amount);
            invoice.payments.set(payer, 0);
            env.storage().instance().set(&invoice_id, &invoice);
        }
        true
    }
    

    // * Get invoice details
    pub fn get_invoice(env: Env, invoice_id: Symbol) -> Invoice {
        env.storage().instance().get(&invoice_id).unwrap_or_else(|| panic!("Invoice not found"))
    }

    // * Verify payment status of an invoice
    pub fn verify_payment(env: Env, invoice_id: Symbol) -> bool {
        let invoice: Invoice = env.storage().instance().get(&invoice_id).unwrap_or_else(|| panic!("Invoice not found"));
        invoice.paid
    }
}

// Invoice structure
#[contracttype]
pub struct Invoice {
    pub total_amount: i128, // * Total amount of the invoice
    pub due_date: u64, // * Due date of the invoice
    pub recipients: Vec<Address>,   // * Recipients of the split payment
    pub amounts: Vec<i128>, // * Amounts to be paid by each recipient
    pub paid: bool, // * Payment status
    pub merchant : Address, // * Merchant address
    pub payments: Map<Address, i128>, // * Payment tracker
}


mod test;
