use crate::hypixel_api::item::ItemData;
use base64::engine::general_purpose;
use base64::Engine;
use fastnbt::stream::{Parser, Value};
use flate2::read::GzDecoder;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_with::serde_as;
use serde_with::TimestampMilliSeconds;
use std::{fmt, io::Read, time::SystemTime};

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Auction {
    pub uuid: String,
    pub auctioneer: String,
    pub profile_id: String,
    pub coop: Vec<String>,
    #[serde_as(as = "TimestampMilliSeconds<i64>")]
    pub start: SystemTime,
    #[serde_as(as = "TimestampMilliSeconds<i64>")]
    pub end: SystemTime,
    pub item_name: String,
    pub item_uuid: Option<String>,
    pub item_lore: String,
    #[serde(rename = "item_bytes", deserialize_with = "item_bytes_to_data")]
    pub item_data: Option<ItemData>,
    pub extra: String,
    pub category: String,
    pub tier: String,
    pub starting_bid: i64,
    pub claimed: bool,
    pub claimed_bidders: Vec<String>,
    pub highest_bid_amount: i64,
    #[serde_as(as = "TimestampMilliSeconds<i64>")]
    pub last_updated: SystemTime,
    pub bin: bool,
    pub bids: Vec<BidEntry>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BidEntry {
    pub auction_id: String,
    pub bidder: String,
    pub profile_id: String,
    pub amount: i64,
    #[serde_as(as = "TimestampMilliSeconds<i64>")]
    pub timestamp: SystemTime,
}

fn item_bytes_to_data<'de, D>(deserializer: D) -> Result<Option<ItemData>, D::Error>
where
    D: Deserializer<'de>,
{
    let so: Option<String> = Deserialize::deserialize(deserializer)?;
    if so.is_none() {
        return Ok(None);
    }
    let s = so.unwrap();
    let decoded_bytes = general_purpose::STANDARD
        .decode(&s)
        .map_err(serde::de::Error::custom)?;
    // Gzip decompress
    let mut gz = GzDecoder::new(&decoded_bytes[..]);
    let mut decompressed_data = Vec::new();
    gz.read_to_end(&mut decompressed_data)
        .map_err(serde::de::Error::custom)?;
    /*
    // Code for debugging / dumping nbt
    println!("{}", s);
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
    let wrap =
        fastnbt::from_bytes::<ItemWrapper>(&decompressed_data).map_err(serde::de::Error::custom)?;
    Ok(Some(wrap.i[0].clone()))
}

#[derive(Serialize, Deserialize)]
struct ItemWrapper {
    pub i: Vec<ItemData>,
}
