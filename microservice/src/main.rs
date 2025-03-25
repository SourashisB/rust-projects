// Financial Operations Microservice
// src/main.rs

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use uuid::Uuid;

// Domain models
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Account {
    id: Uuid,
    customer_id: Uuid,
    account_number: String,
    account_type: AccountType,
    balance: Decimal,
    currency: Currency,
    status: AccountStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "account_type", rename_all = "snake_case")]
enum AccountType {
    Checking,
    Savings,
    Investment,
    CreditCard,
    Loan,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "currency", rename_all = "UPPERCASE")]
enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CAD,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "account_status", rename_all = "snake_case")]
enum AccountStatus {
    Active,
    Inactive,
    Frozen,
    Closed,
    PendingApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    id: Uuid,
    account_id: Uuid,
    transaction_type: TransactionType,
    amount: Decimal,
    reference: String,
    description: Option<String>,
    status: TransactionStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_type", rename_all = "snake_case")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Transfer,
    Payment,
    Fee,
    Interest,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_status", rename_all = "snake_case")]
enum TransactionStatus {
    Pending,
    Completed,
    Failed,
    Reversed,
}

// API DTOs
#[derive(Debug, Clone, Deserialize)]
struct CreateAccountRequest {
    customer_id: Uuid,
    account_type: AccountType,
    currency: Currency,
    initial_deposit: Option<Decimal>,
}

#[derive(Debug, Clone, Deserialize)]
struct TransactionRequest {
    from_account_id: Uuid,
    to_account_id: Option<Uuid>,
    amount: Decimal,
    transaction_type: TransactionType,
    description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ServiceResponse<T> {
    status: String,
    data: Option<T>,
    error: Option<String>,
}

// For rate limiting
struct RateLimiter {
    requests: HashMap<String, Vec<DateTime<Utc>>>,
    limit: usize,
    window_seconds: u64,
}

impl RateLimiter {
    fn new(limit: usize, window_seconds: u64) -> Self {
        RateLimiter {
            requests: HashMap::new(),
            limit,
            window_seconds,
        }
    }

    fn check_rate_limit(&mut self, key: &str) -> bool {
        let now = Utc::now();
        let window_start = now - chrono::Duration::seconds(self.window_seconds as i64);
        
        let requests = self.requests.entry(key.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        requests.retain(|&time| time >= window_start);
        
        // Check if we're over the limit
        if requests.len() >= self.limit {
            return false;
        }
        
        // Add the current request
        requests.push(now);
        true
    }
}

// Database logic
struct DbClient {
    pool: Pool<Postgres>,
}

impl DbClient {
    async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        
        Ok(DbClient { pool })
    }
    
    async fn get_account(&self, id: Uuid) -> Result<Option<Account>, sqlx::Error> {
        sqlx::query_as!(
            Account,
            r#"
            SELECT 
                id, customer_id, account_number, 
                account_type as "account_type: AccountType", 
                balance, 
                currency as "currency: Currency",
                status as "status: AccountStatus",
                created_at, updated_at
            FROM accounts
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }
    
    async fn create_account(&self, req: CreateAccountRequest) -> Result<Account, sqlx::Error> {
        let account_number = format!("ACC-{}", Uuid::new_v4().simple());
        let initial_balance = req.initial_deposit.unwrap_or(Decimal::ZERO);
        let now = Utc::now();
        let id = Uuid::new_v4();
        
        sqlx::query!(
            r#"
            INSERT INTO accounts (
                id, customer_id, account_number, account_type, 
                balance, currency, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            id,
            req.customer_id,
            account_number,
            req.account_type as AccountType,
            initial_balance,
            req.currency as Currency,
            AccountStatus::Active as AccountStatus,
            now,
            now
        )
        .execute(&self.pool)
        .await?;
        
        Ok(Account {
            id,
            customer_id: req.customer_id,
            account_number,
            account_type: req.account_type,
            balance: initial_balance,
            currency: req.currency,
            status: AccountStatus::Active,
            created_at: now,
            updated_at: now,
        })
    }
    
    async fn create_transaction(&self, req: &TransactionRequest) -> Result<Transaction, sqlx::Error> {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let reference = format!("TXN-{}", Uuid::new_v4().simple());
        
        // Begin transaction
        let mut tx = self.pool.begin().await?;
        
        // Update source account balance
        let from_account = sqlx::query_as!(
            Account,
            r#"
            SELECT 
                id, customer_id, account_number, 
                account_type as "account_type: AccountType", 
                balance, 
                currency as "currency: Currency",
                status as "status: AccountStatus",
                created_at, updated_at
            FROM accounts
            WHERE id = $1
            FOR UPDATE
            "#,
            req.from_account_id
        )
        .fetch_one(&mut *tx)
        .await?;
        
        // Check if account has sufficient funds for withdrawal or transfer
        if (req.transaction_type == TransactionType::Withdrawal || 
            req.transaction_type == TransactionType::Transfer) && 
            from_account.balance < req.amount {
            return Err(sqlx::Error::RowNotFound); // Using this as a placeholder for insufficient funds
        }
        
        let new_from_balance = match req.transaction_type {
            TransactionType::Withdrawal | TransactionType::Transfer | TransactionType::Payment | TransactionType::Fee => {
                from_account.balance - req.amount
            },
            TransactionType::Deposit | TransactionType::Interest => {
                from_account.balance + req.amount
            },
        };
        
        sqlx::query!(
            r#"
            UPDATE accounts
            SET balance = $1, updated_at = $2
            WHERE id = $3
            "#,
            new_from_balance,
            now,
            req.from_account_id
        )
        .execute(&mut *tx)
        .await?;
        
        // Handle transfer to destination account if applicable
        if let Some(to_account_id) = req.to_account_id {
            if req.transaction_type == TransactionType::Transfer {
                let to_account = sqlx::query_as!(
                    Account,
                    r#"
                    SELECT 
                        id, customer_id, account_number, 
                        account_type as "account_type: AccountType", 
                        balance, 
                        currency as "currency: Currency",
                        status as "status: AccountStatus",
                        created_at, updated_at
                    FROM accounts
                    WHERE id = $1
                    FOR UPDATE
                    "#,
                    to_account_id
                )
                .fetch_one(&mut *tx)
                .await?;
                
                let new_to_balance = to_account.balance + req.amount;
                
                sqlx::query!(
                    r#"
                    UPDATE accounts
                    SET balance = $1, updated_at = $2
                    WHERE id = $3
                    "#,
                    new_to_balance,
                    now,
                    to_account_id
                )
                .execute(&mut *tx)
                .await?;
            }
        }
        
        // Create transaction record
        sqlx::query!(
            r#"
            INSERT INTO transactions (
                id, account_id, transaction_type, amount,
                reference, description, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            id,
            req.from_account_id,
            req.transaction_type as TransactionType,
            req.amount,
            reference,
            req.description,
            TransactionStatus::Completed as TransactionStatus,
            now,
            now
        )
        .execute(&mut *tx)
        .await?;
        
        // Commit transaction
        tx.commit().await?;
        
        Ok(Transaction {
            id,
            account_id: req.from_account_id,
            transaction_type: req.transaction_type,
            amount: req.amount,
            reference,
            description: req.description.clone(),
            status: TransactionStatus::Completed,
            created_at: now,
            updated_at: now,
        })
    }
    
    async fn get_account_transactions(&self, account_id: Uuid, limit: i64) -> Result<Vec<Transaction>, sqlx::Error> {
        sqlx::query_as!(
            Transaction,
            r#"
            SELECT 
                id, account_id, 
                transaction_type as "transaction_type: TransactionType", 
                amount, reference, description, 
                status as "status: TransactionStatus",
                created_at, updated_at
            FROM transactions
            WHERE account_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            account_id,
            limit
        )
        .fetch_all(&self.pool)
        .await
    }
}

// Service layer
struct AppState {
    db_client: DbClient,
    rate_limiter: Mutex<RateLimiter>,
}

// API handlers
async fn get_account(
    state: web::Data<AppState>,
    path: web::Path<(Uuid,)>,
) -> impl Responder {
    let account_id = path.0;
    
    if !state.rate_limiter.lock().unwrap().check_rate_limit(&account_id.to_string()) {
        return HttpResponse::TooManyRequests().json(ServiceResponse {
            status: "error".to_string(),
            data: None::<String>,
            error: Some("Rate limit exceeded".to_string()),
        });
    }
    
    match state.db_client.get_account(account_id).await {
        Ok(Some(account)) => HttpResponse::Ok().json(ServiceResponse {
            status: "success".to_string(),
            data: Some(account),
            error: None,
        }),
        Ok(None) => HttpResponse::NotFound().json(ServiceResponse {
            status: "error".to_string(),
            data: None::<Account>,
            error: Some("Account not found".to_string()),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ServiceResponse {
            status: "error".to_string(),
            data: None::<Account>,
            error: Some(format!("Database error: {}", e)),
        }),
    }
}

async fn create_account(
    state: web::Data<AppState>,
    req: web::Json<CreateAccountRequest>,
) -> impl Responder {
    if !state.rate_limiter.lock().unwrap().check_rate_limit(&req.customer_id.to_string()) {
        return HttpResponse::TooManyRequests().json(ServiceResponse {
            status: "error".to_string(),
            data: None::<Account>,
            error: Some("Rate limit exceeded".to_string()),
        });
    }
    
    match state.db_client.create_account(req.0).await {
        Ok(account) => HttpResponse::Created().json(ServiceResponse {
            status: "success".to_string(),
            data: Some(account),
            error: None,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ServiceResponse {
            status: "error".to_string(),
            data: None::<Account>,
            error: Some(format!("Failed to create account: {}", e)),
        }),
    }
}

async fn create_transaction(
    state: web::Data<AppState>,
    req: web::Json<TransactionRequest>,
) -> impl Responder {
    if !state.rate_limiter.lock().unwrap().check_rate_limit(&req.from_account_id.to_string()) {
        return HttpResponse::TooManyRequests().json(ServiceResponse {
            status: "error".to_string(),
            data: None::<Transaction>,
            error: Some("Rate limit exceeded".to_string()),
        });
    }
    
    match state.db_client.create_transaction(&req.0).await {
        Ok(transaction) => HttpResponse::Created().json(ServiceResponse {
            status: "success".to_string(),
            data: Some(transaction),
            error: None,
        }),
        Err(sqlx::Error::RowNotFound) => HttpResponse::BadRequest().json(ServiceResponse {
            status: "error".to_string(),
            data: None::<Transaction>,
            error: Some("Insufficient funds or account not found".to_string()),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ServiceResponse {
            status: "error".to_string(),
            data: None::<Transaction>,
            error: Some(format!("Transaction failed: {}", e)),
        }),
    }
}

async fn get_account_transactions(
    state: web::Data<AppState>,
    path: web::Path<(Uuid,)>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let account_id = path.0;
    let limit = query.get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(20);
    
    if !state.rate_limiter.lock().unwrap().check_rate_limit(&account_id.to_string()) {
        return HttpResponse::TooManyRequests().json(ServiceResponse {
            status: "error".to_string(),
            data: None::<Vec<Transaction>>,
            error: Some("Rate limit exceeded".to_string()),
        });
    }
    
    match state.db_client.get_account_transactions(account_id, limit).await {
        Ok(transactions) => HttpResponse::Ok().json(ServiceResponse {
            status: "success".to_string(),
            data: Some(transactions),
            error: None,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ServiceResponse {
            status: "error".to_string(),
            data: None::<Vec<Transaction>>,
            error: Some(format!("Failed to get transactions: {}", e)),
        }),
    }
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file if present
    dotenv::dotenv().ok();
    
    // Configure logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let db_client = DbClient::new(&database_url)
        .await
        .expect("Failed to connect to database");
    
    // Configure rate limiter: 100 requests per minute
    let rate_limiter = Mutex::new(RateLimiter::new(100, 60));
    
    let app_state = web::Data::new(AppState {
        db_client,
        rate_limiter,
    });
    
    // Start HTTP server
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");
    
    log::info!("Starting server on port {}", port);
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/api/v1")
                    .route("/accounts", web::post().to(create_account))
                    .route("/accounts/{id}", web::get().to(get_account))
                    .route("/accounts/{id}/transactions", web::get().to(get_account_transactions))
                    .route("/transactions", web::post().to(create_transaction))
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

// Migration SQL (would be in a separate file)
/*
CREATE TYPE account_type AS ENUM ('checking', 'savings', 'investment', 'credit_card', 'loan');
CREATE TYPE currency AS ENUM ('USD', 'EUR', 'GBP', 'JPY', 'CAD');
CREATE TYPE account_status AS ENUM ('active', 'inactive', 'frozen', 'closed', 'pending_approval');
CREATE TYPE transaction_type AS ENUM ('deposit', 'withdrawal', 'transfer', 'payment', 'fee', 'interest');
CREATE TYPE transaction_status AS ENUM ('pending', 'completed', 'failed', 'reversed');

CREATE TABLE accounts (
    id UUID PRIMARY KEY,
    customer_id UUID NOT NULL,
    account_number TEXT NOT NULL UNIQUE,
    account_type account_type NOT NULL,
    balance DECIMAL(19, 4) NOT NULL,
    currency currency NOT NULL,
    status account_status NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_accounts_customer_id ON accounts(customer_id);

CREATE TABLE transactions (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id),
    transaction_type transaction_type NOT NULL,
    amount DECIMAL(19, 4) NOT NULL,
    reference TEXT NOT NULL,
    description TEXT,
    status transaction_status NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_transactions_account_id ON transactions(account_id);
CREATE INDEX idx_transactions_created_at ON transactions(created_at);
*/

// Cargo.toml
/*
[package]
name = "finance-microservice"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4.3.1"
chrono = { version = "0.4.26", features = ["serde"] }
dotenv = "0.15.0"
env_logger = "0.10.0"
log = "0.4.19"
rust_decimal = { version = "1.30.0", features = ["serde"] }
serde = { version = "1.0.171", features = ["derive"] }
serde_json = "1.0.102"
sqlx = { version = "0.7.1", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "decimal", "json"] }
tokio = { version = "1.29.1", features = ["full"] }
uuid = { version = "1.4.1", features = ["serde", "v4"] }
*/