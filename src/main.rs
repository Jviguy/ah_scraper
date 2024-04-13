use crate::hypixel_api::auction::Auction;
use crate::hypixel_api::{get_all_auctions, get_first_x_pages_of_auctions};
use diesel::upsert::excluded;
use diesel::ExpressionMethods;
use models::Auction as AuctionModel;
use schema::auctions;
use std::error::Error;
use std::time::Duration;
use std::{env, fmt};
use tokio::task;
use tokio::task::JoinError;
use tokio::time;
use tokio::time::Instant;
pub mod hypixel_api;
pub mod models;
pub mod schema;
use diesel_async::{
    pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection, RunQueryDsl,
};

#[tokio::main]
async fn main() -> Result<(), AHScraperError> {
    env_logger::init();
    dotenvy::from_filename(".env.local").or(dotenvy::dotenv())?;
    println!("Starting AH Scraper!");
    println!("Indexing all auctions on the AH to the database.");
    let start = Instant::now();
    let auctions_list = hypixel_api::get_all_auctions().await?;
    println!("Finished scraping all auctions in: {:.2?}", start.elapsed());
    let config =
        AsyncDieselConnectionManager::<AsyncPgConnection>::new(std::env::var("DATABASE_URL")?);
    let pool = Pool::builder().build(config).await?;
    println!("Finished Connecting to db, upserting all data now");
    let start = Instant::now();
    process_auctions_in_parallel(pool.clone(), auctions_list, 100).await?;
    println!("Finished indexing all auctions in: {:.2?}", start.elapsed());
    scrape_task(pool).await?;
    Ok(())
}

async fn scrape_task(db: Pool<AsyncPgConnection>) -> Result<(), AHScraperError> {
    let mut interval = time::interval(Duration::from_millis(100));
    let mut counter = 0;
    loop {
        interval.tick().await;
        // every 10 minutes we should scan all pages as to update on auctions that may have moved.
        if counter > 12000 {
            let db = db.clone();
            tokio::spawn(async move {
                if let Err(e) =
                    process_auctions_in_parallel(db, get_all_auctions().await.unwrap(), 100).await
                {
                    println!("Database operation failed: {}", e);
                }
            });
        } else {
            process_auctions_in_parallel(db.clone(), get_first_x_pages_of_auctions(10).await?, 100)
                .await?;
        };
        counter += 1;
        println!("Finished 500 millisecond update");
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn upsert_auctions(
    db: Pool<AsyncPgConnection>,
    auctions_values: Vec<Auction>,
) -> Result<(), AHScraperError> {
    let dbauctions: Vec<AuctionModel> = auctions_values
        .into_iter()
        .map(AuctionModel::from)
        .collect::<Vec<_>>();
    let mut conn = db.get().await?;
    diesel::insert_into(auctions::table)
        .values(&dbauctions)
        .on_conflict(auctions::uuid)
        .do_update()
        .set((
            auctions::last_updated.eq(excluded(auctions::last_updated)),
            auctions::end_time.eq(excluded(auctions::end_time)),
            auctions::claimed.eq(excluded(auctions::claimed)),
        ))
        .execute(&mut conn)
        .await?;
    Ok(())
}

async fn process_auctions_in_parallel(
    db: Pool<AsyncPgConnection>,
    auctions: Vec<Auction>,
    chunk_size: usize,
) -> Result<(), Box<AHScraperError>> {
    let mut tasks = Vec::new();

    for chunk in auctions.chunks(chunk_size) {
        let chunk = chunk.to_owned().to_vec(); // Clone the chunk data
                                               // Clone the collection handle
        let db_clone = db.clone();
        let task = task::spawn(async move { upsert_auctions(db_clone, chunk).await });

        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await?;
    }
    println!("Finished processing all auctions in parallel");
    Ok(())
}

#[derive(Debug)]
enum AHScraperError {
    Reqwest(reqwest::Error),
    Env(env::VarError),
    DotEnv(dotenvy::Error),
    Join(JoinError),
    Diesel(diesel::result::Error),
    DieselConnection(diesel::ConnectionError),
    DieselPool(diesel_async::pooled_connection::bb8::RunError),
    DieselPoolConnection(diesel_async::pooled_connection::PoolError),
}

impl fmt::Display for AHScraperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AHScraperError::Reqwest(e) => {
                write!(f, "Web Request error (includes parsing of data): {}", e)
            }
            AHScraperError::Env(e) => {
                write!(f, "Env var error (PLEASE SPECIFY CONNECTION_URL): {}", e)
            }
            AHScraperError::DotEnv(e) => {
                write!(f, "DotEnv error (ensure .env.local or .env): {}", e)
            }
            AHScraperError::Join(e) => write!(
                f,
                "Join Error has occured (loop failed to properly stop): {}",
                e
            ),
            AHScraperError::Diesel(e) => write!(f, "Diesel Error: {}", e),
            AHScraperError::DieselConnection(e) => write!(f, "Diesel Connection Error: {}", e),
            AHScraperError::DieselPool(e) => write!(f, "Diesel Pool Error: {}", e),
            AHScraperError::DieselPoolConnection(e) => {
                write!(f, "Diesel Pool Connection Error: {}", e)
            }
        }
    }
}

impl Error for AHScraperError {}

impl From<reqwest::Error> for AHScraperError {
    fn from(error: reqwest::Error) -> Self {
        AHScraperError::Reqwest(error)
    }
}

impl From<env::VarError> for AHScraperError {
    fn from(error: env::VarError) -> Self {
        AHScraperError::Env(error)
    }
}

impl From<dotenvy::Error> for AHScraperError {
    fn from(value: dotenvy::Error) -> Self {
        AHScraperError::DotEnv(value)
    }
}

impl From<JoinError> for AHScraperError {
    fn from(value: JoinError) -> Self {
        AHScraperError::Join(value)
    }
}

impl From<Box<AHScraperError>> for AHScraperError {
    fn from(value: Box<AHScraperError>) -> Self {
        *value
    }
}

impl From<JoinError> for Box<AHScraperError> {
    fn from(value: JoinError) -> Self {
        Box::new(AHScraperError::Join(value))
    }
}

impl From<diesel::result::Error> for AHScraperError {
    fn from(value: diesel::result::Error) -> Self {
        AHScraperError::Diesel(value)
    }
}

impl From<diesel::ConnectionError> for AHScraperError {
    fn from(value: diesel::ConnectionError) -> Self {
        AHScraperError::DieselConnection(value)
    }
}

impl From<diesel_async::pooled_connection::bb8::RunError> for AHScraperError {
    fn from(value: diesel_async::pooled_connection::bb8::RunError) -> Self {
        AHScraperError::DieselPool(value)
    }
}

impl From<diesel_async::pooled_connection::PoolError> for AHScraperError {
    fn from(value: diesel_async::pooled_connection::PoolError) -> Self {
        AHScraperError::DieselPoolConnection(value)
    }
}
