use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use chrono::NaiveDate;

const BLANK_CHAR: char = 'ー';

#[derive(Serialize, Deserialize, Debug)]
pub struct GameList {
    titles: BTreeMap<String, GameTitle>,
}

impl GameList {
    pub fn new() -> Self {
        GameList {
            titles: BTreeMap::new(),
        }
    }
    
    pub fn from_str(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
    
    pub fn from_reader<R: std::io::Read>(reader: R) -> Result<Self, serde_json::Error> {
        serde_json::from_reader(reader)
    }
}

pub fn serialize_title_option<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(title) => serializer.serialize_str(title),
        None => serializer.serialize_str(&BLANK_CHAR.to_string()),
    }
}

pub fn deserialize_title_option<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s == BLANK_CHAR.to_string() {
        Ok(None)
    } else {
        Ok(Some(s))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameTitle {
    /// Game title ID
    pub code: String,
    /// Compatible game title IDs (i.e Different regions, different versions)
    #[serde(default)]
    pub compatible_titles: Option<Vec<String>>,
    /// Extra documentation for this game, as a list of paths
    /// to PNG files
    #[serde(default)]
    pub connect_guides: Option<Vec<String>>,
    /// Copyright information for this game.
    /// Should be in the format of `©<year> <publisher>[/<developers>/]
    /// 
    /// Example: `©2023 Nintendo.`, `©1995-2006 Nintendo/Creatures Inc./GAME FREAK inc.`
    pub copyright: String,
    /// Path to the cover (Box) image of the game.
    pub cover: String,
    /// Path to a detailed image of the game. (Title Screen)
    pub details_screen: String,
    /// Display name for the version of the game. i.e. Region/Revision
    #[serde(default)]
    pub display_version: Option<String>,
    /// Fade-in data
    #[serde(default)]
    pub fadein: Option<[i32; 2]>,
    /// List of countries where this title will be hidden
    #[serde(default)]
    pub hidden_countries: Option<Vec<String>>,
    /// Game re-release date (Date Added in Database)
    pub lcla6_release_date: NaiveDate,
    /// Guide images for the game cartridge
    #[serde(default)]
    pub onecartridge_guides: Option<Vec<String>>,
    /// Maximum player count supported by this title
    pub players_count: i32,
    /// Publisher of the title
    pub publisher: String,
    /// Original release date of the title
    /// 
    /// This value will be a string due to the date format
    /// accepting wildcards such as "2022-01-??", for games without a clear
    /// release date
    pub release_date: String,
    /// How much snapshots to rewind
    pub rewind_interval: f32,
    /// Path to the ROM file of the game.
    pub rom: String,
    /// Maximum save slot count supported by this title
    pub save_count: i32,
    /// Whether this title supports simultaneous multiplayer
    pub simultaneous: bool,
    /// Publisher of the title (for sorting)
    pub sort_publisher: String,
    /// Title of the game (for sorting)
    pub sort_title: String,
    /// Size of the SRAM this title comes with, in bytes
    /// 
    /// Used for emulating the cartridge's SRAM
    #[serde(default)]
    pub sram_file_size: Option<i32>,
    /// Path to a .break format save state file
    #[serde(default)]
    pub startup_state: Option<String>,
    /// Title of the game
    pub title: String,
    /// Title of the game in Korean
    #[serde(default, serialize_with = "serialize_title_option", deserialize_with = "deserialize_title_option")]
    pub title_ko: Option<String>,
    /// Title of the game in Chinese (Simplified)
    #[serde(rename = "title_zhHans", default, serialize_with = "serialize_title_option", deserialize_with = "deserialize_title_option")]
    pub title_zh_hans: Option<String>,
    /// Title of the game in Chinese (Traditional)
    #[serde(rename = "title_zhHant", default, serialize_with = "serialize_title_option", deserialize_with = "deserialize_title_option")]
    pub title_zh_hant: Option<String>,
    /// Audio volume to start this title with
    pub volume: i32,
    /// Adjust game color palette
    #[serde(default)]
    pub adjust_colors: Option<String>,
    /// Alternative guides for this title
    #[serde(default)]
    pub anothertitle_guides: Option<Vec<String>>,
    /// Title ID of the game to transfer data from
    #[serde(default)]
    pub transfer_title: Option<String>,
}


pub fn sanitize_sort_title(title: &str) -> String {
    title.to_lowercase().replace(' ', "_")
}

// #[cfg(test)]
// mod tests {
//     use super::*;
    
//     #[test]
//     fn test_gba() {
//        let gba_manifest = include_str!("../test/gba.json");
       
//         let game_list = GameList::from_str(gba_manifest).unwrap();
//         println!("{:#?}", game_list.titles);
        
//         // // serialize back to json
//         // let json = serde_json::to_string(&game_list).unwrap();
//         // println!("{}", json);
//     }
    
//     #[test]
//     fn test_snes() {
//        let gba_manifest = include_str!("../test/snes.json");
       
//        println!("gba_manifest: {}", gba_manifest);
       
//         let game_list = GameList::from_str(gba_manifest).unwrap();
//         println!("{:#?}", game_list.titles);
        
//         // // serialize back to json
//         // let json = serde_json::to_string(&game_list).unwrap();
//         // println!("{}", json);
//     }
// }