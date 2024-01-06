use serde::{Deserialize, Deserializer, Serialize};
use base64::engine::general_purpose;
use flate2::read::GzDecoder;
use std::io::Read;
use base64::Engine;
use fastnbt::stream::{Parser, Value};
use crate::hypixel_api::item::ItemData;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Auction {
    pub uuid: String,
    pub auctioneer: String,
    pub profile_id: String,
    pub coop: Vec<String>,
    pub start: u64,
    pub end: u64,
    pub item_name: String,
    pub item_uuid: Option<String>,
    pub item_lore: String,
    #[serde(rename = "item_bytes", deserialize_with = "item_bytes_to_data")]
    pub item_data: ItemData,
    pub extra: String,
    pub category: String,
    pub tier: String,
    pub starting_bid: u64,
    pub claimed: bool,
    pub claimed_bidders: Vec<String>,
    pub highest_bid_amount: u64,
    pub last_updated: u64,
    pub bin: bool,
    pub bids: Vec<BidEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BidEntry {
    pub auction_id: String,
    pub bidder: String,
    pub profile_id: String,
    pub amount: u64,
    pub timestamp: u64
}

fn item_bytes_to_data<'de, D>(deserializer: D) -> Result<ItemData, D::Error>
    where
        D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let decoded_bytes = general_purpose::STANDARD.decode(&s).map_err(serde::de::Error::custom)?;
    // Gzip decompress
    let mut gz = GzDecoder::new(&decoded_bytes[..]);
    let mut decompressed_data = Vec::new();
    gz.read_to_end(&mut decompressed_data).map_err(serde::de::Error::custom)?;
    /*
    Code for debugging / dumping nbt
    println!("{}",s);
    gz = GzDecoder::new(&decoded_bytes[..]);
    let mut parser = Parser::new(gz);
    let mut indent = 0;
    loop {
        match parser.next() {
            Err(e) => {
                break;
            }
            Ok(value) => {
                match value {
                    Value::CompoundEnd => indent -= 4,
                    Value::ListEnd => indent -= 4,
                    _ => {}
                }

                println!("{:indent$}{:?}", "", value, indent = indent);

                match value {
                    Value::Compound(_) => indent += 4,
                    Value::List(_, _, _) => indent += 4,
                    _ => {}
                }
            }
        }
    }
    */
    // Deserialize the decompressed data into ItemData
    let wrap = fastnbt::from_bytes::<ItemWrapper>(&decompressed_data).map_err(serde::de::Error::custom)?;
    return Ok(wrap.i[0].clone())

}

#[derive(Serialize, Deserialize)]
struct ItemWrapper {
    pub i: Vec<ItemData>,
}