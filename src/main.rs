use std::error::Error;
use mongodb::{bson, Client, Collection, Database};
use dotenv;
use std::{env, fmt};
use std::time::Duration;
use mongodb::bson::doc;
use mongodb::options::UpdateOptions;
use tokio::task;
use tokio::task::JoinError;
use tokio::time;
use tokio::time::Instant;
use crate::hypixel_api::auction::Auction;
use crate::hypixel_api::{get_all_auctions, get_first_x_pages_of_auctions};

mod hypixel_api;

#[tokio::main]
async fn main() -> Result<(), AHScraperError> {
    env_logger::init();
    dotenv::from_filename(".env.local").or(dotenv::dotenv())?;
    println!("Starting AH Scraper!");
    println!("Indexing all auctions on the AH to the database.");
    let start = Instant::now();
    let auctions = hypixel_api::get_all_auctions().await?;
    println!("Finished scraping all auctions in: {:.2?}", start.elapsed());
    let client = Client::with_uri_str(
        format!("mongodb+srv://{}:{}@ahdatabase0.dfdbtmh.mongodb.net/?retryWrites=true&w=majority&authMechanism=SCRAM-SHA-1",
                env::var("MONGODB_USERNAME")?,
                env::var("MONGODB_PASSWORD")?
        )
    ).await?;
    let db = client.database("auction_house");
    let coll = db.collection::<Auction>("auctions");
    println!("Finished Connecting to db, upserting all data now");
    let start = Instant::now();
    process_auctions_in_parallel(coll, auctions, 100).await?;
    println!("Finished indexing all auctions in: {:.2?}", start.elapsed());
    let handle = task::spawn_blocking(move || {
        futures::executor::block_on(scrape_task(db))
    });
    Ok(())
}

async fn scrape_task(db: Database) -> Result<(), AHScraperError> {
    let mut interval = time::interval(Duration::from_millis(100));
    let mut counter = 0;
    loop {
        interval.tick().await;
        // every 10 minutes we should scan all pages as to update on auctions that may have moved.
        let auctions = if counter > 12000 {
            get_all_auctions().await?

        } else {
            get_first_x_pages_of_auctions(10).await?
        };
        upsert_auctions(db.collection("auctions"), auctions).await?;
        counter += 1;
    }
    Ok(())
}

async fn upsert_auctions(coll: Collection<Auction>, auctions: Vec<Auction>) -> Result<(), mongodb::error::Error> {
    for auction in auctions {
        // Create or update each document
        // Assume `doc` has a unique identifier field, e.g., "id"
        let filter = doc! { "uuid": &auction.uuid };
        let update = doc! { "$set": bson::to_bson(&auction)? };
        let options = UpdateOptions::builder().upsert(true).build();
        let start = Instant::now();
        coll.update_one(filter, update, options).await?;
        println!("Finished uploading one auction in: {:.2?}", start.elapsed());
    }
    Ok(())
}

async fn process_auctions_in_parallel(coll: Collection<Auction>, auctions: Vec<Auction>, chunk_size: usize) -> Result<(), Box<AHScraperError>> {
    let mut tasks = Vec::new();

    for chunk in auctions.chunks(chunk_size) {
        let chunk = chunk.to_owned().to_vec(); // Clone the chunk data
        let coll_clone = coll.clone(); // Clone the collection handle

        let task = task::spawn(async move {
            upsert_auctions(coll_clone, chunk).await
        });

        tasks.push(task);
    }

    for task in tasks {
        task.await??;
    }

    Ok(())
}

#[derive(Debug)]
enum AHScraperError {
    ReqwestError(reqwest::Error),
    EnvError(env::VarError),
    MongoDBError(mongodb::error::Error),
    DotEnvError(dotenv::Error),
    JoinError(JoinError)
}

impl fmt::Display for AHScraperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AHScraperError::ReqwestError(e) => write!(f, "Web Request error (includes parsing of data): {}", e),
            AHScraperError::EnvError(e) => write!(f, "Env var error (PLEASE SPECIFY MONGODB_USERNAME and MONGODB_PASSWORD): {}", e),
            AHScraperError::MongoDBError(e) => write!(f, "Mongodb error (ensure database status): {}", e),
            AHScraperError::DotEnvError(e) => write!(f, "DotEnv error (ensure .env.local or .env): {}", e),
            AHScraperError::JoinError(e) => write!(f, "Join Error has occured (loop failed to properly stop): {}", e),
        }
    }
}

impl Error for AHScraperError {}

impl From<reqwest::Error> for AHScraperError {
    fn from(error: reqwest::Error) -> Self {
        AHScraperError::ReqwestError(error)
    }
}

impl From<env::VarError> for AHScraperError {
    fn from(error: env::VarError) -> Self {
        AHScraperError::EnvError(error)
    }
}

impl From<mongodb::error::Error> for AHScraperError {
    fn from(value: mongodb::error::Error) -> Self {
        AHScraperError::MongoDBError(value)
    }
}

impl From<dotenv::Error> for AHScraperError {
    fn from(value: dotenv::Error) -> Self {
        AHScraperError::DotEnvError(value)
    }
}

impl From<JoinError> for AHScraperError {
    fn from(value: JoinError) -> Self {
        AHScraperError::JoinError(value)
    }
}

impl From<Box<AHScraperError>> for AHScraperError {
    fn from(value: Box<AHScraperError>) -> Self {
        *value
    }
}

impl From<JoinError> for Box<AHScraperError> {
    fn from(value: JoinError) -> Self {
        Box::new(AHScraperError::JoinError(value))
    }
}

impl From<mongodb::error::Error>  for Box<AHScraperError> {
    fn from(value: mongodb::error::Error) -> Self {
        Box::new(AHScraperError::MongoDBError(value))
    }
}