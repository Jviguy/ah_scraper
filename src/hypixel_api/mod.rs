use std::error::Error;
use crate::hypixel_api::auction::Auction;
use crate::hypixel_api::page::Page;

pub mod page;
pub mod auction;
pub mod item;


pub async fn get_page(page: u32) -> Result<Page, reqwest::Error> {
    reqwest::get(format!("https://api.hypixel.net/v2/skyblock/auctions?page={}", page)).await?.json::<Page>().await
}

pub async fn get_all_auctions() -> Result<Vec<Auction>, reqwest::Error> {
    let first_page = get_page(0).await?;
    let mut v = first_page.auctions;
    for i in 1..first_page.total_pages {
        v.append(&mut get_page(i).await?.auctions);
    }
    return Ok(v)
}

pub async fn get_first_x_pages_of_auctions(x: u32) -> Result<Vec<Auction>, reqwest::Error> {
    let mut v = vec![];
    for i in 0..x {
        v.append(&mut get_page(i).await?.auctions);
    }
    return Ok(v)
}