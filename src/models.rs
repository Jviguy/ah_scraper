use std::time::SystemTime;

use crate::hypixel_api::{
    auction::Auction as APIAuction,
    item::{Gem, MinecraftEnchant},
};
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::auctions)]
pub struct Auction {
    pub uuid: String,
    pub auctioneer: String,
    pub profile_id: String,
    pub coop: Option<Vec<String>>,
    pub start_time: SystemTime,
    pub end_time: SystemTime,
    pub item_name: String,
    pub item_uuid: Option<String>,
    pub item_lore: Option<String>,
    pub item_id: Option<String>,
    pub item_count: Option<i32>,
    pub item_damage: Option<i32>,
    pub enchantments: Option<Vec<String>>,
    pub unbreakable: Option<bool>,
    pub price: i64,
    pub claimed: Option<bool>,
    pub tier: String,
    pub category: String,
    pub last_updated: SystemTime,
    pub bin: bool,
    pub reforge: Option<String>,
    pub upgrade_level: Option<i32>,
    pub hot_potato_count: Option<i32>,
    pub recomb: Option<bool>,
    pub unlocked_gem_slots: Option<Vec<String>>,
    pub slotted_gems: Option<Vec<String>>,
    pub pet_active: Option<bool>,
    pub pet_type: Option<String>,
    pub pet_held_item: Option<String>,
    pub pet_exp: Option<i32>,
    pub pet_candy_used: Option<i32>,
    pub dungeon_item_level: Option<i32>,
    pub red_armor_coloring: Option<i32>,
    pub green_armor_coloring: Option<i32>,
    pub blue_armor_coloring: Option<i32>,
    pub anvil_uses: Option<i32>,
    pub pelts_earned: Option<i32>,
    pub champion_combat_xp: Option<i32>,
    pub compact_blocks: Option<i32>,
    pub hecatomb_s_runs: Option<i32>,
    pub expertise_kills: Option<i32>,
    pub runes: Option<Vec<String>>,
    pub farmed_cultivating: Option<i32>,
}

impl From<APIAuction> for Auction {
    fn from(value: APIAuction) -> Self {
        let mut slotted_gems = None;
        let mut unlocked_gems_slots: Option<Vec<String>> = None;
        let mut enchs: Option<Vec<String>> = None;
        let mut r = None;
        let mut g = None;
        let mut b = None;
        let mut runes = None;
        let mut pet_active = None;
        let mut pet_type = None;
        let mut pet_held_item = None;
        let mut pet_exp = None;
        let mut pet_candy_used = None;
        let mut item_id = None;
        let mut item_count = None;
        let mut item_damage = None;
        let mut unbreakable = None;
        let mut reforge = None;
        let mut upgrade_level = None;
        let mut hot_potato_count = None;
        let mut recomb = None;
        let mut dungeon_item_level = None;
        let mut anvil_uses = None;
        let mut pelts_earned = None;
        let mut champion_combat_xp = None;
        let mut farmed_cultivating = None;
        let mut compact_blocks = None;
        let mut hecatomb_s_runs = None;
        let mut expertise_kills = None;
        if let Some(item) = value.item_data {
            if let Some(gems) = item.tag.extra_attributes.gems {
                slotted_gems = Some(
                    gems.iter()
                        .filter(|(_name, gem)| {
                            matches!(gem, &Gem::SlottedGem(_))
                                || matches!(gem, &Gem::SlottedGemStruct(_))
                        })
                        .map(|(name, gem)| {
                            if let Gem::SlottedGem(gem) = gem {
                                gem.clone()
                            } else if let Gem::SlottedGemStruct(gem) = gem {
                                format!("{} {}", name, gem.quality)
                            } else {
                                panic!("Gem is not a slotted gem")
                            }
                        })
                        .collect(),
                );
                let temp = gems
                    .iter()
                    .filter(|(_name, gem)| matches!(gem, &Gem::UnlockedSlot(_)))
                    .collect::<Vec<(&String, &Gem)>>();
                let x = temp.first();
                if let Some((_name, gem)) = x {
                    if let Gem::UnlockedSlot(slots) = gem {
                        unlocked_gems_slots = Some(slots.clone());
                    }
                };
            }

            if let Some(ench) = item.tag.extra_attributes.enchantments {
                enchs = Some(
                    ench.iter()
                        .map(|(name, enchd)| format!("{} {}", name, enchd))
                        .collect(),
                );
            }
            if let Some(color) = item.tag.extra_attributes.color {
                let split: Vec<i32> = color
                    .split(':')
                    .map(|x| x.parse::<i32>().unwrap())
                    .collect();
                r = Some(split[0]);
                g = Some(split[1]);
                b = Some(split[2]);
            }
            if let Some(rune) = item.tag.extra_attributes.runes {
                runes = Some(
                    rune.iter()
                        .map(|(name, level)| format!("{} {}", name, level))
                        .collect(),
                );
            }

            if let Some(pet_info) = item.tag.extra_attributes.pet_info {
                pet_active = pet_info.active;
                pet_type = Some(pet_info.p_type);
                pet_held_item = pet_info.held_item;
                pet_exp = Some(pet_info.exp as i32);
                pet_candy_used = pet_info.candy_used.map(|x| x as i32);
            }
            item_id = Some(item.tag.extra_attributes.id);
            item_count = Some(item.count as i32);
            item_damage = Some(item.damage as i32);
            unbreakable = item.tag.unbreakable;
            reforge = item.tag.extra_attributes.modifier;
            upgrade_level = item.tag.extra_attributes.upgrade_level.map(|x| x as i32);
            hot_potato_count = item.tag.extra_attributes.hot_potato_count.map(|x| x as i32);
            recomb = item.tag.extra_attributes.rarity_upgrades;
            dungeon_item_level = item
                .tag
                .extra_attributes
                .dungeon_item_level
                .map(|x| x as i32);
            anvil_uses = item.tag.extra_attributes.anvil_uses.map(|x| x as i32);
            pelts_earned = item.tag.extra_attributes.pelts_earned.map(|x| x as i32);
            champion_combat_xp = item
                .tag
                .extra_attributes
                .champion_combat_xp
                .map(|x| x as i32);
            farmed_cultivating = item
                .tag
                .extra_attributes
                .farmed_cultivating
                .map(|x| x as i32);
            compact_blocks = item.tag.extra_attributes.compact_blocks.map(|x| x as i32);
            hecatomb_s_runs = item.tag.extra_attributes.hecatomb_s_runs.map(|x| x as i32);
            expertise_kills = item.tag.extra_attributes.expertise_kills.map(|x| x as i32);
        }
        Auction {
            uuid: value.uuid,
            auctioneer: value.auctioneer,
            profile_id: value.profile_id,
            coop: Some(value.coop),
            start_time: value.start,
            end_time: value.end,
            item_name: value.item_name,
            item_uuid: value.item_uuid,
            item_lore: Some(value.item_lore),
            item_id,
            item_count,
            item_damage,
            enchantments: enchs,
            unbreakable,
            price: value.starting_bid,
            claimed: Some(value.claimed),
            tier: value.tier,
            category: value.category,
            last_updated: value.last_updated,
            bin: value.bin,
            reforge,
            upgrade_level,
            hot_potato_count,
            recomb,
            unlocked_gem_slots: unlocked_gems_slots,
            slotted_gems,
            pet_active,
            pet_type,
            pet_held_item,
            pet_exp,
            pet_candy_used,
            dungeon_item_level,
            red_armor_coloring: r,
            green_armor_coloring: g,
            blue_armor_coloring: b,
            anvil_uses,
            pelts_earned,
            champion_combat_xp,
            farmed_cultivating,
            compact_blocks,
            hecatomb_s_runs,
            expertise_kills,
            runes,
        }
    }
}
