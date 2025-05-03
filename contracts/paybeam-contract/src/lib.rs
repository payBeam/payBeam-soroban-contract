#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, log, Address, Env, Map, Symbol, Vec};
use soroban_sdk::token::TokenClient;
#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    
    pub fn create_invoice(
        env: Env,
        invoice_id: Symbol, // Unique invoice ID
        total_amount: i128,
        due_date: u64,
        merchant: Address,
        memo: Symbol,
    ) -> Symbol {
        merchant.require_auth();

        //  To ensure the invoice ID is unique
        if env.storage().instance().has(&invoice_id) {
            panic!("Invoice ID already exists");
        }

        // To validate the split payment amounts
        // if recipients.len() != amounts.len() {
        //     panic!("Recipients and amounts must have the same length");
        // }

        if total_amount <= 0 {
            panic!("Amount must be positive");
        }

        if due_date <= env.ledger().timestamp() {
            panic!("Due date must be in the future");
        }

        // let total_split_amount: i128 = amounts.iter().sum();
        // if total_split_amount != total_amount {
        //     panic!("Total split amount must equal the invoice amount");
        // }

        let memo_for_key = memo.clone(); 

        // let token_address: Address = env.storage().persistent()
        //     .get(&Symbol::new(&env, "token"))
        //     .unwrap_or_else(|| panic!("Token not initialized"));

        // invoice details
        let invoice = Invoice {
            total_amount,
            due_date,
            recipients: Vec::new(&env), // Add the merchant as a recipient
            merchant,
            amounts: Vec::new(&env),
            paid: false,
            payments: Map::new(&env),
            // token: token_address.clone(),
            memo,
        };


        env.storage().instance().set(&memo_for_key, &invoice_id);

        // Save the invoice
        env.storage().instance().set(&invoice_id, &invoice);

        log!(&env, "Invoice created", invoice_id);
        // Return the id of the created invoice
        invoice_id
    }

    pub fn get_invoice_by_memo(env: Env, memo: Symbol) -> Option<Invoice> {
        let maybe_invoice_id: Option<Symbol> = env.storage().instance().get(&memo);
        match maybe_invoice_id {
            Some(invoice_id) => env.storage().instance().get(&invoice_id),
            None => None,
        }
    }
    

    // Pay a portion of the invoice
    pub fn pay_invoice(
        env: Env,
        invoice_id: Symbol,
        payer: Address,
        amount: i128,
    ) {
        payer.require_auth();
        // Fetch the invoice
        let mut invoice: Invoice = env.storage().instance().get(&invoice_id).unwrap_or_else(|| panic!("Invoice not found"));

        // Check if the invoice is already paid
        if invoice.paid {
            // return false;
            panic!("Invoice is already paid");
        }

        if env.ledger().timestamp() > invoice.due_date {
            panic!("Invoice expired");
        } 

        // let addr : Address = Address::from_str("USDC_CONTRACT_ADDRESS").unwrap();

        // Transfer USDC from payer to the escrow contract
        let token_client = TokenClient::new(&env, &Address::from_str(&env, "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA")); // USDC testnet contract address
        token_client.transfer(&payer, &env.current_contract_address(), &amount);

        // Update the payment tracker
        let total_paid = invoice.payments.get(payer.clone()).unwrap_or(0) + amount;
        invoice.payments.set(payer.clone(), total_paid);

        // Check if the invoice is fully paid
        let total_payments: i128 = invoice.payments.values().into_iter().sum();
        if total_payments >= invoice.total_amount {
            invoice.paid = true;

            // Overpayment refund
            if total_payments > invoice.total_amount {
                let overpayment = total_payments - invoice.total_amount;
                token_client.transfer(&env.current_contract_address(), &payer.clone(), &overpayment);
                log!(&env, "Overpayment refunded", (payer, overpayment));
            }

            // release funds to the merchant once fully paid and overpayments are sorted.
            Self::release_funds(env.clone(), invoice_id.clone());
        }

        // Save the updated invoice
        env.storage().instance().set(&invoice_id, &invoice);
        // log!(&env, "Payment received", (payer, amount));
    }

    // Release funds to merchant once the invoice is fully paid
    fn release_funds(env: Env, invoice_id: Symbol) {
        let invoice: Invoice = env.storage().instance().get(&invoice_id).unwrap_or_else(|| panic!("Invoice not found"));

        // Ensure the invoice is fully paid
        if !invoice.paid {
            panic!("Invoice is not fully paid");
        }

        // Transfer funds to merchant when fully paid
        let token_client = TokenClient::new(&env, &Address::from_str(&env, "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"));
        token_client.transfer(&env.current_contract_address(), &invoice.merchant, &invoice.total_amount);
    }

    

    pub fn expire_invoice(env: Env, invoice_id: Symbol) -> bool {
        let mut invoice: Invoice = env.storage().instance().get(&invoice_id).unwrap_or_else(|| panic!("Invoice not found"));
    
        // Check if the invoice is already paid or expired
        if invoice.paid || env.ledger().timestamp() > invoice.due_date {
            return false;
        }
    
        // Mark the invoice as expired
        invoice.paid = true; 
        env.storage().instance().set(&invoice_id, &invoice);
        true
    }

    // Refund a payment
    pub fn refund_payment(env: Env, invoice_id: Symbol, payer: Address) -> bool {
        let mut invoice: Invoice = env.storage().instance().get(&invoice_id).unwrap_or_else(|| panic!("Invoice not found"));
    
        // Ensure the invoice is expired
        if !invoice.paid || env.ledger().timestamp() <= invoice.due_date {
            return false;
        }
    
        // Refund the payer's contribution
        let amount = invoice.payments.get(payer.clone()).unwrap_or(0);
        if amount > 0 {
            let token_client = TokenClient::new(&env, &Address::from_str(&env, "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"));
            token_client.transfer(&env.current_contract_address(), &payer, &amount);
            invoice.payments.set(payer, 0);
            env.storage().instance().set(&invoice_id, &invoice);
        }
        true
    }
    

    // Get invoice details
    pub fn get_invoice(env: Env, invoice_id: Symbol) -> Invoice {
        env.storage().instance().get(&invoice_id).unwrap_or_else(|| panic!("Invoice not found"))
    }

    // Verify payment status of an invoice
    pub fn verify_payment(env: Env, invoice_id: Symbol) -> bool {
        let invoice: Invoice = env.storage().instance().get(&invoice_id).unwrap_or_else(|| panic!("Invoice not found"));
        invoice.paid
    }
}

// Invoice structure
#[contracttype]
pub struct Invoice {
    pub total_amount: i128, // Total amount of the invoice
    pub due_date: u64, // Due date of the invoice
    pub recipients: Vec<Address>,   // Recipients of the split payment
    pub amounts: Vec<i128>, // Amounts to be paid by each recipient
    pub paid: bool, // Payment status
    pub merchant : Address, // Merchant address
    // pub token : Address, // Token contract address for payments
    pub payments: Map<Address, i128>, // Payment tracker
    pub memo : Symbol, // Memo for the invoice
}

mod test;
