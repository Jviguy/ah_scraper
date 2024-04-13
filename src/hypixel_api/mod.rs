use crate::hypixel_api::auction::Auction;
use crate::hypixel_api::page::Page;
use std::error::Error;
use std::thread::sleep;
use std::time::Instant;

pub mod auction;
pub mod item;
pub mod page;

pub async fn get_page(page: u32) -> Result<Page, reqwest::Error> {
    //let start = Instant::now();
    let response = reqwest::get(format!(
        "https://api.hypixel.net/v2/skyblock/auctions?page={}",
        page
    ))
    .await?;
    //println!("Time for request: {:?}", start.elapsed());
    //let start = Instant::now();
    response.json::<Page>().await
    //println!("Time for json: {:?}", start.elapsed());
}

pub async fn get_all_auctions() -> Result<Vec<Auction>, reqwest::Error> {
    let first_page = get_page(0).await?;
    let mut v = first_page.auctions;
    for i in 1..first_page.total_pages {
        v.append(&mut get_page(i).await?.auctions);
    }
    Ok(v)
}

pub async fn get_first_x_pages_of_auctions(x: u32) -> Result<Vec<Auction>, reqwest::Error> {
    let mut v = vec![];
    for i in 0..x {
        v.append(&mut get_page(i).await?.auctions);
    }
    Ok(v)
}
