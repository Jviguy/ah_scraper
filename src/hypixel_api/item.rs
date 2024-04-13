use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemData {
    pub id: u32,
    #[serde(rename = "Count")]
    pub count: u8,
    pub tag: Tag,
    #[serde(rename = "Damage")]
    pub damage: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Tag {
    #[serde(rename = "ench")]
    pub ench: Option<Vec<MinecraftEnchant>>,
    pub unbreakable: Option<bool>,
    pub hide_flags: Option<u8>,
    #[serde(rename = "display")]
    pub display: DisplayInfo,
    pub extra_attributes: ExtraAttributes,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftEnchant {
    pub lvl: u16,
    pub id: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DisplayInfo {
    pub lore: Vec<String>,
    #[serde(rename = "color")]
    pub color: Option<u32>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum HypixelTimestamp {
    Timestamp(i64),
    TimeStamp(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtraAttributes {
    pub id: String,
    pub uuid: Option<String>,
    pub timestamp: Option<HypixelTimestamp>,
    pub rarity_upgrades: Option<bool>,
    pub modifier: Option<String>,
    pub upgrade_level: Option<u8>,
    pub hot_potato_count: Option<u8>,
    // these ones are very weird.
    pub enchantments: Option<HashMap<String, u8>>,
    pub gems: Option<HashMap<String, Gem>>,
    // end weird stuff.
    pub dungeon_item_level: Option<u8>,
    pub dungeon_item: Option<bool>,
    #[serde(rename = "originTag")]
    pub origin_tag: Option<String>,
    pub color: Option<String>,
    pub anvil_uses: Option<u32>,
    pub pelts_earned: Option<u32>,
    pub champion_combat_xp: Option<f32>,
    pub farmed_cultivating: Option<u32>,
    pub compact_blocks: Option<u32>,
    pub hecatomb_s_runs: Option<u32>,
    pub expertise_kills: Option<u32>,
    pub farming_for_dummies_count: Option<u32>,
    pub runes: Option<HashMap<String, u8>>,
    #[serde(default)]
    #[serde(rename = "petInfo", deserialize_with = "pet_json_to_struct")]
    pub pet_info: Option<PetInfo>,
    pub raffle_year: Option<u32>,
    pub raffle_win: Option<String>,
    pub dye_item: Option<String>,
}

fn pet_json_to_struct<'de, D>(deserializer: D) -> Result<Option<PetInfo>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(Some(
        serde_json::from_str::<PetInfo>(&s).map_err(serde::de::Error::custom)?,
    ))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Gem {
    UnlockedSlot(Vec<String>),
    SlottedGem(String),
    SlottedGemStruct(SlottedGem),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SlottedGem {
    pub uuid: String,
    pub quality: String,
}

// This is a json object stored in the string petInfo in the nbt. Don't ask me man.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PetInfo {
    #[serde(rename = "type")]
    pub p_type: String,
    pub active: Option<bool>,
    pub held_item: Option<String>,
    pub exp: f32,
    pub candy_used: Option<u8>,
    pub tier: Option<String>,
    pub skin: Option<String>,
    pub uuid: Option<String>,
    pub unique_id: Option<String>,
    pub hide_right_click: Option<bool>,
    pub no_move: Option<bool>,
    pub hide_info: Option<bool>,
}
