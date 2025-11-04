#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec};

// Data structures
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subscription {
    pub id: u64,
    pub owner: Address,
    pub subscriber: Address,
    pub amount: i128,
    pub period_days: u32,
    pub recipient: Address,
    pub split_percentage: u32, // Basis points (0-10000, where 10000 = 100%)
    pub next_billing_date: u64,
    pub last_payment_date: u64,
    pub is_active: bool,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipientBalance {
    pub address: Address,
    pub balance: i128,
}

// Storage keys
#[contracttype]
pub enum DataKey {
    Subscription(u64),
    OwnerSubscriptions(Address),
    RecipientBalance(Address),
    SubscriptionCounter,
    Initialized,
}

const MAX_SPLIT_PERCENTAGE: u32 = 10000; // 100% in basis points

#[contract]
pub struct SubSor;

#[contractimpl]
impl SubSor {
    /// Initialize the contract (one-time setup)
    pub fn initialize(env: Env) {
        // Check if already initialized
        if env.storage().instance().has(&DataKey::Initialized) {
            return;
        }
        // Initialize counter and mark as initialized
        env.storage().instance().set(&DataKey::SubscriptionCounter, &0u64);
        env.storage().instance().set(&DataKey::Initialized, &true);
    }

    /// Create a new subscription
    pub fn create_subscription(
        env: Env,
        owner: Address,
        subscriber: Address,
        amount: i128,
        period_days: u32,
        recipient: Address,
        split_percentage: u32,
    ) -> u64 {
        owner.require_auth();
        
        // Validate inputs
        if amount <= 0 {
            panic!("Amount must be positive");
        }
        if period_days == 0 {
            panic!("Period must be at least 1 day");
        }
        if split_percentage > MAX_SPLIT_PERCENTAGE {
            panic!("Split percentage cannot exceed 100%");
        }

        // Increment subscription counter
        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::SubscriptionCounter)
            .unwrap_or(0);
        counter = counter.checked_add(1).unwrap();
        env.storage().instance().set(&DataKey::SubscriptionCounter, &counter);

        // Calculate next billing date (current ledger timestamp + period in seconds)
        let current_time = env.ledger().timestamp();
        let period_seconds = (period_days as u64).checked_mul(86400).unwrap();
        let next_billing = (current_time as u64).checked_add(period_seconds).unwrap();

        let subscription = Subscription {
            id: counter,
            owner: owner.clone(),
            subscriber: subscriber.clone(),
            amount,
            period_days,
            recipient: recipient.clone(),
            split_percentage,
            next_billing_date: next_billing as u64,
            last_payment_date: 0,
            is_active: true,
            created_at: current_time as u64,
        };

        // Store subscription
        env.storage().instance().set(&DataKey::Subscription(counter), &subscription);

        // Add to owner's subscription list
        let mut owner_subs: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::OwnerSubscriptions(owner.clone()))
            .unwrap_or(Vec::new(&env));
        owner_subs.push_back(counter);
        env.storage().instance().set(&DataKey::OwnerSubscriptions(owner), &owner_subs);

        // Initialize recipient balance if needed
        if !env.storage().instance().has(&DataKey::RecipientBalance(recipient.clone())) {
            env.storage().instance().set(&DataKey::RecipientBalance(recipient), &0i128);
        }

        counter
    }

    /// Cancel an active subscription
    pub fn cancel_subscription(env: Env, subscription_id: u64) {
        let subscription: Subscription = env
            .storage()
            .instance()
            .get(&DataKey::Subscription(subscription_id))
            .unwrap_or_else(|| panic!("Subscription not found"));
        
        subscription.owner.require_auth();
        
        if !subscription.is_active {
            panic!("Subscription already cancelled");
        }

        let mut cancelled_sub = subscription;
        cancelled_sub.is_active = false;
        env.storage().instance().set(&DataKey::Subscription(subscription_id), &cancelled_sub);
    }

    /// Renew a subscription (can be called by anyone when due)
    pub fn renew_subscription(env: Env, subscription_id: u64) -> bool {
        let subscription: Subscription = env
            .storage()
            .instance()
            .get(&DataKey::Subscription(subscription_id))
            .unwrap_or_else(|| panic!("Subscription not found"));

        if !subscription.is_active {
            panic!("Subscription is not active");
        }

        let current_time = env.ledger().timestamp() as u64;
        
        if current_time < subscription.next_billing_date {
            return false; // Not yet due
        }

        // Transfer payment from subscriber to contract (in a real implementation, this would use token transfers)
        // For this example, we'll just update balances and dates
        
        // Calculate split amounts
        let split_amount = (subscription.amount as u128)
            .checked_mul(subscription.split_percentage as u128)
            .and_then(|x| x.checked_div(MAX_SPLIT_PERCENTAGE as u128))
            .unwrap_or(0) as i128;

        // Update recipient balance
        let recipient_balance: i128 = env
            .storage()
            .instance()
            .get(&DataKey::RecipientBalance(subscription.recipient.clone()))
            .unwrap_or(0);
        let new_balance = recipient_balance.checked_add(split_amount).unwrap();
        env.storage().instance().set(
            &DataKey::RecipientBalance(subscription.recipient.clone()),
            &new_balance,
        );

        // Update subscription dates
        let mut renewed_sub = subscription;
        renewed_sub.last_payment_date = current_time;
        let period_seconds = (renewed_sub.period_days as u64).checked_mul(86400).unwrap();
        renewed_sub.next_billing_date = current_time
            .checked_add(period_seconds)
            .unwrap();
        
        env.storage().instance().set(&DataKey::Subscription(subscription_id), &renewed_sub);

        true
    }

    /// Get subscription details
    pub fn get_subscription(env: Env, subscription_id: u64) -> Subscription {
        env.storage()
            .instance()
            .get(&DataKey::Subscription(subscription_id))
            .unwrap_or_else(|| panic!("Subscription not found"))
    }

    /// Withdraw accumulated revenue for a recipient
    pub fn withdraw_revenue(env: Env, recipient: Address) -> i128 {
        recipient.require_auth();
        
        let balance: i128 = env
            .storage()
            .instance()
            .get(&DataKey::RecipientBalance(recipient.clone()))
            .unwrap_or(0);

        if balance <= 0 {
            return 0;
        }

        // Reset balance (in real implementation, would transfer tokens here)
        env.storage().instance().set(&DataKey::RecipientBalance(recipient), &0i128);

        balance
    }

    /// Get balance for a recipient
    pub fn get_balance(env: Env, recipient: Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::RecipientBalance(recipient))
            .unwrap_or(0)
    }

    /// List subscriptions for an owner with pagination
    pub fn list_subscriptions(
        env: Env,
        owner: Address,
        start_after: Option<u64>,
        limit: u32,
    ) -> Vec<Subscription> {
        let subscription_ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::OwnerSubscriptions(owner))
            .unwrap_or(Vec::new(&env));

        let mut result = Vec::new(&env);
        let mut found_start = start_after.is_none();
        let mut count = 0u32;

        for id in subscription_ids.iter() {
            if !found_start {
                if start_after.is_some() && id == start_after.unwrap() {
                    found_start = true;
                }
                continue;
            }

            if count >= limit {
                break;
            }

            if let Some(sub) = env.storage().instance().get::<DataKey, Subscription>(&DataKey::Subscription(id)) {
                result.push_back(sub);
                count += 1;
            }
        }

        result
    }

    /// Get all subscriptions for an owner
    pub fn get_all_subscriptions(env: Env, owner: Address) -> Vec<Subscription> {
        let subscription_ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::OwnerSubscriptions(owner))
            .unwrap_or(Vec::new(&env));

        let mut result = Vec::new(&env);

        for id in subscription_ids.iter() {
            if let Some(sub) = env.storage().instance().get::<DataKey, Subscription>(&DataKey::Subscription(id)) {
                result.push_back(sub);
            }
        }

        result
    }

    /// Check and auto-renew all due subscriptions (helper function)
    pub fn process_due_subscriptions(env: Env, owner: Address, max_count: u32) -> u32 {
        let subscription_ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::OwnerSubscriptions(owner))
            .unwrap_or(Vec::new(&env));

        let mut renewed = 0u32;
        let current_time = env.ledger().timestamp() as u64;

        for id in subscription_ids.iter() {
            if renewed >= max_count {
                break;
            }

            if let Some(sub) = env.storage().instance().get::<DataKey, Subscription>(&DataKey::Subscription(id)) {
                if sub.is_active && current_time >= sub.next_billing_date {
                    // Renew this subscription
                    let _ = Self::renew_subscription(env.clone(), id);
                    renewed += 1;
                }
            }
        }

        renewed
    }
}

#[cfg(test)]
mod test;
