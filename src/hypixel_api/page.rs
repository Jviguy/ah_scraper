use crate::hypixel_api::auction::Auction;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub success: bool,
    pub page: u32,
    pub total_pages: u32,
    pub total_auctions: u32,
    pub last_updated: u64,
    pub auctions: Vec<Auction>,
}
