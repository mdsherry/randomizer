use std::{io::Error, path::Path, collections::{HashMap, HashSet}, hash::Hash};
use allocator::{ConditionFactory, ItemCategory, Condition, allocate};
use heck::ToSnakeCase;
mod logic_parse;
mod header;
mod allocator;

macro_rules! cond_or {
    ($($e:expr),*) => {
        Condition::Or(vec![$((Condition::from($e))),*])
    };
}
macro_rules! cond_and {
    ($($e:expr),*) => {
        Condition::And(vec![$(Condition::from($e)),*])
    };
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileEntityType {
    None = 0x00,
    TestA = 0x01,
    Chest = 0x02,
    BigChest = 0x03,
    TestB = 0x04,
    TestC = 0x05,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RomVersion {
    EU,
    JP,
    US,
    Unknown
}
struct Rom {
    buf: Vec<u8>,
    version: RomVersion
}

impl Rom {
    fn new(fname: &Path) -> Result<Self, std::io::Error> {
        let buf = std::fs::read(fname)?;
        let version = match &buf[0xAC..0xAC + 4] {
            b"BZMP" => RomVersion::EU,
            b"BZMJ" => RomVersion::JP,
            b"BZME" => RomVersion::US,
            _ => RomVersion::Unknown
        };
        Ok(Self { buf, version })
    }
    fn read_at<const N: usize>(&self, pos: usize) -> [u8; N] {
        let mut rv = [0; N];
        rv[..N].clone_from_slice(&self.buf[pos..(N + pos)]);
        rv
    }
    
}
use logic_parse::parse_logic;
fn main() {
    // Rom::new(Path::new("foo.rom"));
    

    // parse_logic(include_str!("default.logic.txt"));
    // return;
    let mut cond_fact = ConditionFactory::new();
    let hyrulean_bestiary = cond_fact.add_item("HyruleanBestiary", ItemCategory::Major);
    let rupee1 = cond_fact.add_item("Rupee1", ItemCategory::Minor);
    let ocarina = cond_fact.add_item("Ocarina", ItemCategory::Major);
    let grip_ring = cond_fact.add_item("GripRing", ItemCategory::Major);
    let power_bracelets = cond_fact.add_item("PowerBracelets", ItemCategory::Major);
    let spin_attack = cond_fact.add_item("SpinAttack", ItemCategory::Major);
    let dash_attack = cond_fact.add_item("DashAttack", ItemCategory::Major);
    let small_key_fow_set_4 = cond_fact.add_item("SmallKey`FOW_SET`:4", ItemCategory::DungeonItem);
    let peril_beam = cond_fact.add_item("PerilBeam", ItemCategory::Major);
    let small_key_pow_set = cond_fact.add_item("SmallKey`POW_SET`", ItemCategory::DungeonItem);
    let bomb_bag = cond_fact.add_item("BombBag", ItemCategory::Major);
    let long_spin = cond_fact.add_item("LongSpin", ItemCategory::Major);
    let bottle2 = cond_fact.add_item("Bottle2", ItemCategory::Major);
    let bow = cond_fact.add_item("Bow", ItemCategory::Major);
    let pacci_cane = cond_fact.add_item("PacciCane", ItemCategory::Major);
    let rupee20 = cond_fact.add_item("Rupee20", ItemCategory::Minor);
    let carlov_medal = cond_fact.add_item("CarlovMedal", ItemCategory::Major);
    let gust_jar = cond_fact.add_item("GustJar", ItemCategory::Major);
    let magic_boomerang = cond_fact.add_item("MagicBoomerang", ItemCategory::Major);
    let bottle4 = cond_fact.add_item("Bottle4", ItemCategory::Major);
    let kinstone_x_yellow_crown = cond_fact.add_item("KinstoneX.YellowCrown", ItemCategory::Major);
    let big_key_cof_set = cond_fact.add_item("BigKey`COF_SET`", ItemCategory::DungeonItem);
    let big_key_dhc_set = cond_fact.add_item("BigKey`DHC_SET`", ItemCategory::DungeonItem);
    let rupee50 = cond_fact.add_item("Rupee50", ItemCategory::Minor);
    let rupee100 = cond_fact.add_item("Rupee100", ItemCategory::Minor);
    let red_sword = cond_fact.add_item("RedSword", ItemCategory::Major);
    let fast_split = cond_fact.add_item("FastSplit", ItemCategory::Major);
    let jabber_nut = cond_fact.add_item("JabberNut", ItemCategory::Major);
    let small_key_pow_set_4 = cond_fact.add_item("SmallKey`POW_SET`:4", ItemCategory::DungeonItem);
    let small_key_dhc_set = cond_fact.add_item("SmallKey`DHC_SET`", ItemCategory::DungeonItem);
    let small_key_dhc_set_1 = cond_fact.add_item("SmallKey`DHC_SET`:1", ItemCategory::DungeonItem);
    let small_key_pow_set_6 = cond_fact.add_item("SmallKey`POW_SET`:6", ItemCategory::DungeonItem);
    let rupee200 = cond_fact.add_item("Rupee200", ItemCategory::Minor);
    let big_key_dws_set = cond_fact.add_item("BigKey`DWS_SET`", ItemCategory::DungeonItem);
    let rupee5 = cond_fact.add_item("Rupee5", ItemCategory::Minor);
    let mole_mitts = cond_fact.add_item("MoleMitts", ItemCategory::Major);
    let light_arrow = cond_fact.add_item("LightArrow", ItemCategory::Major);
    let small_key_dws_set = cond_fact.add_item("SmallKey`DWS_SET`", ItemCategory::DungeonItem);
    let water_element = cond_fact.add_item("WaterElement", ItemCategory::Major);
    let kinstone_x_yellow_totem_prong = cond_fact.add_item("KinstoneX.YellowTotemProng", ItemCategory::Major);
    let down_thrust = cond_fact.add_item("DownThrust", ItemCategory::Major);
    let graveyard_key = cond_fact.add_item("GraveyardKey", ItemCategory::Major);
    let kinstone_x_yellow_tornado_prong = cond_fact.add_item("KinstoneX.YellowTornadoProng", ItemCategory::Major);
    let bottle1 = cond_fact.add_item("Bottle1", ItemCategory::Major);
    let mask_history = cond_fact.add_item("MaskHistory", ItemCategory::Major);
    let wallet = cond_fact.add_item("Wallet", ItemCategory::Major);
    let four_sword = cond_fact.add_item("FourSword", ItemCategory::Major);
    let green_sword = cond_fact.add_item("GreenSword", ItemCategory::Major);
    let fast_spin = cond_fact.add_item("FastSpin", ItemCategory::Major);
    let great_spin = cond_fact.add_item("GreatSpin", ItemCategory::Major);
    let roll_attack = cond_fact.add_item("RollAttack", ItemCategory::Major);
    let untyped_0x_ff = cond_fact.add_item("Untyped.0xFF", ItemCategory::Major);
    let bottle3 = cond_fact.add_item("Bottle3", ItemCategory::Major);
    let flippers = cond_fact.add_item("Flippers", ItemCategory::Major);
    let small_key_cof_set = cond_fact.add_item("SmallKey`COF_SET`", ItemCategory::DungeonItem);
    let boomerang = cond_fact.add_item("Boomerang", ItemCategory::Major);
    let dog_food_bottle = cond_fact.add_item("DogFoodBottle", ItemCategory::Major);
    let big_key_tod_set = cond_fact.add_item("BigKey`TOD_SET`", ItemCategory::DungeonItem);
    let smith_sword = cond_fact.add_item("SmithSword", ItemCategory::Major);
    let shield = cond_fact.add_item("Shield", ItemCategory::Major);
    let sword_beam = cond_fact.add_item("SwordBeam", ItemCategory::Major);
    let small_key_rc_set_3 = cond_fact.add_item("SmallKey`RC_SET`:3", ItemCategory::DungeonItem);
    let fire_element = cond_fact.add_item("FireElement", ItemCategory::Major);
    let earth_element = cond_fact.add_item("EarthElement", ItemCategory::Major);
    let heart_container = cond_fact.add_item("HeartContainer", ItemCategory::Minor);
    let small_key_tod_set_4 = cond_fact.add_item("SmallKey`TOD_SET`:4", ItemCategory::Major);
    let rocs_cape = cond_fact.add_item("RocsCape", ItemCategory::Major);
    let picori_legend = cond_fact.add_item("PicoriLegend", ItemCategory::Major);
    let small_key_cof_set_2 = cond_fact.add_item("SmallKey`COF_SET`:2", ItemCategory::DungeonItem);
    let small_key_dws_set_4 = cond_fact.add_item("SmallKey`DWS_SET`:4", ItemCategory::DungeonItem);
    let small_key_dws_set_2 = cond_fact.add_item("SmallKey`DWS_SET`:2", ItemCategory::DungeonItem);
    let wind_element = cond_fact.add_item("WindElement", ItemCategory::Major);
    let tingle_trophy = cond_fact.add_item("TingleTrophy", ItemCategory::Major);
    let big_key_fow_set = cond_fact.add_item("BigKey`FOW_SET`", ItemCategory::DungeonItem);
    let rock_breaker = cond_fact.add_item("RockBreaker", ItemCategory::Major);
    let lantern_off = cond_fact.add_item("LanternOff", ItemCategory::Major);
    let piece_of_heart = cond_fact.add_item("PieceOfHeart", ItemCategory::Minor);
    let big_key_pow_set = cond_fact.add_item("BigKey`POW_SET`", ItemCategory::DungeonItem);
    let lon_lon_key = cond_fact.add_item("LonLonKey", ItemCategory::Major);
    let pegasus_boots = cond_fact.add_item("PegasusBoots", ItemCategory::Major);
    let wake_up_mushroom = cond_fact.add_item("WakeUpMushroom", ItemCategory::Major);
    let blue_sword = cond_fact.add_item("BlueSword", ItemCategory::Major);


    
    // let has_sword = cond_fact.add_flag("HasSword", cond_or!(smith_sword, green_sword, red_sword, blue_sword, four_sword));
    // let can_split_2 = cond_fact.add_flag("HasSword", cond_and!(smith_sword, green_sword, red_sword));
    // let can_split_3 = cond_fact.add_flag("HasSword", cond_and!(smith_sword, green_sword, red_sword, blue_sword));
    // let can_split_4 = cond_fact.add_flag("HasSword", cond_and!(smith_sword, green_sword, red_sword, blue_sword, four_sword));
    // let has_bottle = cond_fact.add_flag("HasBottle", cond_or!(bottle1, bottle2, bottle3, bottle4));
    // let has_spin = cond_fact.add_flag("HasBottle", cond_or!(spin_attack, fast_spin, fast_split, great_spin, long_spin));
    // let has_damage_source = cond_fact.add_flag("HasDamageSource", cond_or!(has_sword, bomb_bag, bow));
    // let can_destroy_trees = cond_fact.add_flag("CanDestroyTrees", cond_or!(has_sword, light_arrow, bomb_bag, lantern_off));
    // let access_east_field = cond_fact.add_flag("AccessEastField", cond_or!(can_destroy_trees, ocarina));
    // let access_west_field = cond_fact.add_flag("AccessWestField", cond_or!(rocs_cape, cond_and!(has_sword, has_spin), flippers, cond_and!(can_split_3, bomb_bag)));
    // let access_hylia_north = cond_fact.add_flag("AccessHyliaNorth", cond_and!(access_east_field, cond_or!(rocs_cape, lon_lon_key, ocarina, cond_and!(flippers, mole_mitts))));
    // let access_hylia_south = cond_fact.add_flag("AccessHyliaSouth", cond_and!(access_hylia_north, cond_or!(flippers, rocs_cape, cond_and!(mole_mitts, pacci_cane))));
    // let access_minish_woods = cond_fact.add_flag("AccessMinishWoods", access_east_field);
    // let access_crenel = cond_fact.add_flag("AccessCrenel", cond_and!(access_west_field, has_bottle, cond_or!(bomb_bag, grip_ring)));
    // let access_lower_crenel = cond_fact.add_flag("AccessCrenel", cond_and!(access_west_field, has_bottle, cond_or!(bomb_bag, cond_and!(rocs_cape, gust_jar))));
    // let deepwood_access = cond_fact.add_flag("DeepwoodAccess", cond_and!(access_minish_woods, cond_or!(jabber_nut, flippers)));
    // let complete_deepwood = cond_fact.add_flag("DeepwoodAccess", cond_and!(deepwood_access, gust_jar, has_sword));


    let flag_has_sword = cond_fact.add_flag("HasSword", cond_or!(smith_sword, green_sword, red_sword, blue_sword, four_sword));
    let flag_has_spin = cond_fact.add_flag("HasSpin", cond_or!(spin_attack, fast_spin, fast_split, great_spin, long_spin));
    let flag_can_split2 = cond_fact.add_flag("CanSplit2", cond_and!(cond_or!(smith_sword, red_sword, blue_sword, four_sword), flag_has_spin));
    let flag_can_split3 = cond_fact.add_flag("CanSplit3", cond_and!(cond_or!(smith_sword, blue_sword, four_sword), flag_has_spin));
    let flag_can_split4 = cond_fact.add_flag("CanSplit4", cond_and!(cond_or!(smith_sword, four_sword), flag_has_spin));
    let flag_has_bottle = cond_fact.add_flag("HasBottle", cond_or!(bottle1, bottle2, bottle3, bottle4));
    let flag_has_bow = cond_fact.add_flag("HasBow", cond_or!(bow, light_arrow));
    let flag_has_light_bow = cond_fact.add_flag("HasLightBow", cond_or!(bow, light_arrow));
    let flag_has_boomerang = cond_fact.add_flag("HasBoomerang", cond_or!(boomerang, magic_boomerang));
    let flag_has_damage_source = cond_fact.add_flag("HasDamageSource", cond_or!(flag_has_sword, flag_has_bow, bomb_bag));
    let flag_has_beam = cond_fact.add_flag("HasBeam", cond_and!(flag_has_sword, cond_or!(sword_beam, peril_beam)));
    let flag_can_down_thrust = cond_fact.add_flag("CanDownThrust", cond_and!(flag_has_sword, down_thrust, rocs_cape));
    let flag_can_destroy_trees = cond_fact.add_flag("CanDestroyTrees", cond_or!(flag_has_sword, flag_has_light_bow, bomb_bag, lantern_off));
    let flag_inaccessible = cond_fact.add_flag("Inaccessible", untyped_0x_ff);
    let flag_access_east_field = cond_fact.add_flag("AccessEastField", cond_or!(flag_can_destroy_trees, ocarina));
    let flag_access_west_field = cond_fact.add_flag("AccessWestField", cond_or!(rocs_cape, cond_and!(flag_has_sword, flag_has_spin), flippers, cond_and!(bomb_bag, flag_can_split3)));
    let flag_access_minish_woods = cond_fact.add_flag("AccessMinishWoods", flag_access_east_field);
    let flag_access_crenel = cond_fact.add_flag("AccessCrenel", cond_and!(flag_access_west_field, flag_has_bottle, cond_or!(bomb_bag, grip_ring)));
    let flag_access_lower_crenel = cond_fact.add_flag("AccessLowerCrenel", cond_and!(flag_access_west_field, flag_has_bottle, cond_or!(bomb_bag, cond_and!(rocs_cape, gust_jar))));
    // let flag_got_scrolls = cond_fact.add_flag("GotScrolls", todo!());
    let flag_access_wilds = cond_fact.add_flag("AccessWilds", cond_and!(flag_access_west_field, flag_can_split2, cond_or!(pegasus_boots, rocs_cape)));
    let flag_access_hylia_north = cond_fact.add_flag("AccessHyliaNorth", cond_and!(flag_access_east_field, cond_or!(rocs_cape, lon_lon_key, ocarina, cond_and!(flippers, mole_mitts))));
    let flag_access_hylia_south = cond_fact.add_flag("AccessHyliaSouth", cond_and!(flag_access_hylia_north, cond_or!(flippers, rocs_cape, cond_and!(mole_mitts, pacci_cane))));
    let flag_access_treasure_cave = cond_fact.add_flag("AccessTreasureCave", cond_and!(flag_access_hylia_north, mole_mitts, rocs_cape));
    let flag_access_valley = cond_fact.add_flag("AccessValley", cond_and!(flag_can_split3, cond_or!(bomb_bag, rocs_cape, flippers), lantern_off));
    let flag_access_crypt = cond_fact.add_flag("AccessCrypt", cond_and!(flag_access_valley, graveyard_key, flag_can_split3, pegasus_boots));
    let flag_access_falls_north = cond_fact.add_flag("AccessFallsNorth", cond_and!(bomb_bag, kinstone_x_yellow_crown, lantern_off));
    let flag_access_falls_south = cond_fact.add_flag("AccessFallsSouth", cond_and!(flag_access_east_field, pacci_cane));
    let flag_access_clouds = cond_fact.add_flag("AccessClouds", cond_and!(flag_access_falls_north, grip_ring));
    let flag_access_upper_clouds = cond_fact.add_flag("AccessUpperClouds", cond_and!(flag_access_clouds, cond_or!(rocs_cape, mole_mitts), kinstone_x_yellow_tornado_prong));
    let flag_deepwood_access = cond_fact.add_flag("DeepwoodAccess:Deepwood", cond_and!(flag_access_minish_woods, cond_or!(jabber_nut, flippers)));
    let flag_complete_deepwood = cond_fact.add_flag("CompleteDeepwood:Deepwood", cond_and!(flag_deepwood_access, gust_jar, flag_has_sword, big_key_dws_set));
    let flag_co_f_access = cond_fact.add_flag("CoFAccess:FlameCave", cond_and!(flag_access_crenel, cond_or!(pacci_cane, cond_and!(grip_ring, cond_or!(rocs_cape, flag_has_light_bow, cond_and!(gust_jar, cond_or!(bomb_bag, flag_has_bow, flag_has_boomerang, flag_has_beam))))), cond_or!(bomb_bag, shield, pacci_cane, flag_can_down_thrust), flag_has_damage_source));
    let flag_complete_co_f = cond_fact.add_flag("CompleteCoF:FlameCave", cond_and!(flag_co_f_access, pacci_cane, small_key_cof_set_2, big_key_cof_set));
    let flag_access_fortress = cond_fact.add_flag("AccessFortress:Fortress", cond_and!(flag_access_wilds, kinstone_x_yellow_totem_prong, cond_or!(flag_has_bow, rocs_cape, flippers)));
    let flag_complete_fortress = cond_fact.add_flag("CompleteFortress:Fortress", cond_and!(flag_access_fortress, mole_mitts, flag_can_split2, flag_has_bow, big_key_fow_set));
    let flag_access_droplets = cond_fact.add_flag("AccessDroplets:Droplets", cond_and!(flag_access_hylia_south, cond_or!(flippers, rocs_cape)));
    let flag_droplets_bottom_jump = cond_fact.add_flag("DropletsBottomJump:Droplets", cond_and!(lantern_off, rocs_cape, flag_has_damage_source, small_key_tod_set_4));
    let flag_droplets_east_lever = cond_fact.add_flag("DropletsEastLever:Droplets", cond_and!(flag_access_droplets, big_key_tod_set, cond_or!(flag_droplets_bottom_jump, cond_and!(gust_jar, flippers, small_key_tod_set_4)), flag_can_split2));
    let flag_droplets_west_lever = cond_fact.add_flag("DropletsWestLever:Droplets", cond_and!(flag_access_droplets, big_key_tod_set, lantern_off, flippers, bomb_bag, small_key_tod_set_4, flag_can_split2));
    let flag_complete_droplets = cond_fact.add_flag("CompleteDroplets:Droplets", cond_and!(flag_droplets_east_lever, flag_droplets_west_lever, lantern_off, flag_has_sword));
    let flag_access_palace = cond_fact.add_flag("AccessPalace:Palace", cond_and!(flag_access_upper_clouds, flag_can_split3, cond_or!(rocs_cape, bomb_bag, gust_jar, flag_has_boomerang, flag_has_bow)));
    let flag_complete_palace = cond_fact.add_flag("CompletePalace:Palace", cond_and!(flag_access_palace, rocs_cape, pacci_cane, lantern_off));
    let flag_dhc_access = cond_fact.add_flag("DHCAccess:DHC", Condition::NoRequirements);
    let flag_beat_vaati = cond_fact.add_flag("BeatVaati", cond_and!(big_key_dhc_set, small_key_dhc_set, flag_can_split4, bomb_bag, flag_has_bow, rocs_cape, lantern_off, gust_jar, pacci_cane));
    let flag_castle_big_doors_open = cond_fact.add_flag("CastleBigDoorsOpen:DHC", cond_and!(flag_dhc_access, small_key_dhc_set_1, flag_can_split4, rocs_cape, cond_or!(boomerang, flag_has_bow), bomb_bag));    
    
    // let smith_house = cond_fact.add_location("SmithHouse", Condition::NoRequirements);
    // let intro_item_1 = cond_fact.add_location("IntroItem1", Condition::NoRequirements);
    // let intro_item_2 = cond_fact.add_location("IntroItem2", Condition::NoRequirements);
    // let hyrule_well_top = cond_fact.add_location("HyruleWellTop", bomb_bag);
    // let hyrule_well_left = cond_fact.add_location("HyruleWellLeft", mole_mitts);
    // let hyrule_well_right = cond_fact.add_location("HyruleWellRight", Condition::NoRequirements);
    // let hyrule_well_bottom = cond_fact.add_location("HyruleWellBottom", cond_or!(flippers, rocs_cape));
    // let hyrule_well_pillar = cond_fact.add_location("HyruleWellPillar", cond_and!(hyrule_well_left, hyrule_well_right, hyrule_well_bottom));
    // let pre_castle_cave_heart_piece = cond_fact.add_location("PreCastleCaveHeartPiece", cond_or!(flippers, rocs_cape, bomb_bag));
    // let swiftblade_scroll_1 = cond_fact.add_location("SwiftbladeScroll1", has_sword);
    // // let swiftblade_scroll_2 = cond_fact.add_location("SwiftbladeScroll2", Condition::Flag(has_sword2));
    // let swiftblade_scroll_3 = cond_fact.add_location("SwiftbladeScroll3", cond_and!(has_sword, pegasus_boots));
    // let swiftblade_scroll_4 = cond_fact.add_location("SwiftbladeScroll4", cond_and!(has_sword, rocs_cape));
    // let grimblade_heart_piece = cond_fact.add_location("GrimbladeHeartPiece", Condition::NoRequirements);
    // let grimblade_scroll = cond_fact.add_location("GrimbladeScroll", cond_and!(has_sword, lantern_off));
    // let castle_water_left = cond_fact.add_location("CastleWaterLeft", flippers);
    // let castle_water_right = cond_fact.add_location("CastleWaterRight", flippers);
    // let cafe_lady = cond_fact.add_location("CafeLady", Condition::NoRequirements);
    // let hearth_ledge = cond_fact.add_location("HearthLedge", lantern_off);
    // let hearth_backdoor = cond_fact.add_location("HearthBackdoor", cond_or!(flippers, pacci_cane, rocs_cape));
    // let school_top = cond_fact.add_location("SchoolTop", pacci_cane);
    // let school_garden_left = cond_fact.add_location("SchoolGardenLeft", cond_and!(pacci_cane, can_split_4));
    // let school_garden_middle = cond_fact.add_location("SchoolGardenMiddle", cond_and!(pacci_cane, can_split_4));
    // let school_garden_right = cond_fact.add_location("SchoolGardenRight", cond_and!(pacci_cane, can_split_4));
    // let school_garden_heart_piece = cond_fact.add_location("SchoolGardenHeartPiece", cond_and!(pacci_cane, can_split_4));
    // let town_digging_top = cond_fact.add_location("TownDiggingTop", mole_mitts);
    // let town_digging_right = cond_fact.add_location("TownDiggingRight", mole_mitts);
    // let town_digging_left = cond_fact.add_location("TownDiggingLeft", mole_mitts);
    // let bakery_attic = cond_fact.add_location("BakeryAttic", cond_or!(pacci_cane, flippers, rocs_cape));
    // let stockwell_attic = cond_fact.add_location("StockwellAttic", cond_or!(pacci_cane, flippers, rocs_cape));
    // let simulation_chest = cond_fact.add_location("SimulationChest", has_sword);
    // let rem_shoe_shop = cond_fact.add_location("RemShoeShop", wake_up_mushroom);
    // let figurine_house_left = cond_fact.add_location("FigurineHouseLeft", carlov_medal);
    // let figurine_house_middle = cond_fact.add_location("FigurineHouseMiddle", carlov_medal);
    // let figurine_house_right = cond_fact.add_location("FigurineHouseRight", carlov_medal);
    // let figurine_house_heart_piece = cond_fact.add_location("FigurineHouseHeartPiece", carlov_medal);
    // let julietta_book = cond_fact.add_location("JuliettaBook", cond_and!(has_bottle, cond_or!(flippers, pacci_cane, rocs_cape)));
    // let wright_attic_book = cond_fact.add_location("WrightAtticBook", cond_and!(has_bottle, cond_or!(flippers, pacci_cane, rocs_cape), cond_or!(gust_jar, bomb_bag), can_split_2));
    // let fountain_big = cond_fact.add_location("FountainBig", cond_and!(has_bottle, pacci_cane, has_damage_source));
    // let fountain_small = cond_fact.add_location("FountainSmall", cond_and!(has_bottle, cond_or!(flippers, rocs_cape)));
    // let fountain_heart_piece = cond_fact.add_location("FountainHeartPiece", cond_and!(has_bottle, rocs_cape));
    // let library_minish = cond_fact.add_location("LibraryMinish", cond_and!(hyrulean_bestiary, picori_legend, mask_history, ocarina, pacci_cane));
    // let town_bell = cond_fact.add_location("TownBell", rocs_cape);
    // let flips_cave_big = cond_fact.add_location("FlipsCaveBig", cond_and!(ocarina, has_damage_source, pacci_cane, cond_or!(flippers, cond_and!(hyrulean_bestiary, picori_legend, mask_history, grip_ring, cond_or!(gust_jar, rocs_cape)))));
    // let flips_cave_small = cond_fact.add_location("FlipsCaveSmall", cond_and!(flippers, ocarina, pacci_cane, lantern_off));
    // let tingle_trophy_item = cond_fact.add_location("TingleTrophyItem", cond_and!(can_destroy_trees, pacci_cane, tingle_trophy));
    
    // let hills_keese_cave = cond_fact.add_location("HillsKeeseCave", bomb_bag);
    // let above_hp_hole = cond_fact.add_location("AboveHPHole", cond_and!(access_hylia_north, cond_or!(pacci_cane, rocs_cape)));
    // let lon_lon_pot = cond_fact.add_location("LonLonPot", access_east_field);
    // let lon_lon_cave = cond_fact.add_location("LonLonCave", cond_and!(access_hylia_north, can_split_2));
    // let lon_lon_cave_secret = cond_fact.add_location("LonLonCaveSecret", cond_and!(lon_lon_cave, bomb_bag, lantern_off));
    // let lon_lon_heart_piece = cond_fact.add_location("LonLonHeartPiece", cond_and!(access_hylia_north, pegasus_boots));
    // let minish_rupee_fairy = cond_fact.add_location("MinishRupeeFairy", cond_and!(access_east_field, pacci_cane));

    // let trilby_bomb_cave = cond_fact.add_location("TrilbyBombCave", cond_and!(access_west_field, can_split_2, bomb_bag));
    // let trilby_mole_cave_left = cond_fact.add_location("TrilbyMoleCaveLeft", cond_and!(access_west_field, mole_mitts));
    // let trilby_mole_cave_right = cond_fact.add_location("TrilbyMoleCaveRight", cond_and!(access_west_field, mole_mitts));
    // let bottle_scrub = cond_fact.add_location("BottleScrub", cond_and!(shield, bomb_bag, access_west_field));
    
    // let jabber_nut_loc = cond_fact.add_location("JabberNut", access_minish_woods);
    // let belari_bombs = cond_fact.add_location("BelariBombs", cond_and!(access_minish_woods, cond_or!(complete_deepwood, bomb_bag)));
    // let minish_middle_flipper_hole = cond_fact.add_location("MinishMiddleFlipperHole", cond_and!(access_minish_woods, flippers, cond_or!(complete_deepwood, bomb_bag)));
    // let minish_right_flipper_hole = cond_fact.add_location("MinishRightFlipperHole", cond_and!(access_minish_woods, cond_or!(complete_deepwood, bomb_bag)));
    // let minish_left_flipper_hole = cond_fact.add_location("MinishLeftFlipperHole", cond_and!(access_minish_woods, cond_or!(complete_deepwood, bomb_bag)));
    // let minish_left_flipper_hole_heart_piece = cond_fact.add_location("MinishLeftFlipperHoleHeartPiece", cond_and!(access_minish_woods, flippers, cond_or!(complete_deepwood, bomb_bag)));
    // let minish_like_like_digging_cave_left = cond_fact.add_location("MinishLikeLikeDiggingCaveLeft", cond_and!(access_minish_woods, mole_mitts));
    // let minish_like_like_digging_cave_right = cond_fact.add_location("MinishLikeLikeDiggingCaveRight", cond_and!(access_minish_woods, mole_mitts));
    // let minish_north_hole = cond_fact.add_location("MinishNorthHole", cond_and!(access_hylia_south, flippers, pegasus_boots));
    // let minish_witch_hut = cond_fact.add_location("MinishWitchHut", cond_and!(access_minish_woods, cond_or!(flippers, rocs_cape, cond_and!(pacci_cane, cond_or!(ocarina, lon_lon_key)))));
    // let minish_heart_piece_top = cond_fact.add_location("MinishHeartPieceTop", cond_and!(access_minish_woods, cond_or!(flippers, rocs_cape, cond_and!(pacci_cane, cond_or!(ocarina, lon_lon_key)))));
    // let minish_heart_piece_bottom = cond_fact.add_location("MinishHeartPieceBottom", access_minish_woods);
    // let minish_village_heart_piece = cond_fact.add_location("MinishVillageHeartPiece", access_minish_woods);


    // let loc_ped_item1 = cond_fact.add_location("PedItem1", todo!());
    // let loc_ped_item2 = cond_fact.add_location("PedItem2", todo!());
    // let loc_ped_item3 = cond_fact.add_location("PedItem3", todo!());
    let loc_smith_house = cond_fact.add_location("SmithHouse", Condition::NoRequirements);
    let loc_intro_item1 = cond_fact.add_location("IntroItem1", Condition::NoRequirements);
    let loc_intro_item2 = cond_fact.add_location("IntroItem2", Condition::NoRequirements);
    let loc_link_minish_water_hole_heart_piece = cond_fact.add_location("LinkMinishWaterHoleHeartPiece", cond_and!(flag_can_destroy_trees, pegasus_boots, flippers));
    let loc_hyrule_well_top = cond_fact.add_location("HyruleWellTop", bomb_bag);
    let loc_hyrule_well_left = cond_fact.add_location("HyruleWellLeft", mole_mitts);
    let loc_hyrule_well_bottom = cond_fact.add_location("HyruleWellBottom", cond_or!(flippers, rocs_cape));
    let loc_hyrule_well_right = cond_fact.add_location("HyruleWellRight", Condition::NoRequirements);
    let loc_hyrule_well_pillar = cond_fact.add_location("HyruleWellPillar", cond_and!(loc_hyrule_well_left, loc_hyrule_well_right, loc_hyrule_well_bottom, flag_can_split3));    
    let loc_pre_castle_cave_heart_piece = cond_fact.add_location("PreCastleCaveHeartPiece", cond_or!(flippers, rocs_cape, bomb_bag));
    let loc_swiftblade_scroll1 = cond_fact.add_location("SwiftbladeScroll1", flag_has_sword);
    let loc_swiftblade_scroll2 = cond_fact.add_location("SwiftbladeScroll2", smith_sword);
    let loc_swiftblade_scroll3 = cond_fact.add_location("SwiftbladeScroll3", cond_and!(flag_has_sword, pegasus_boots));
    let loc_swiftblade_scroll4 = cond_fact.add_location("SwiftbladeScroll4", cond_and!(flag_has_sword, rocs_cape));
    let loc_grimblade_heart_piece = cond_fact.add_location("GrimbladeHeartPiece", Condition::NoRequirements);
    let loc_grimblade_scroll = cond_fact.add_location("GrimbladeScroll", cond_and!(flag_has_sword, lantern_off));
    let loc_castle_water_left = cond_fact.add_location("CastleWaterLeft", flippers);
    let loc_castle_water_right = cond_fact.add_location("CastleWaterRight", flippers);
    let loc_cafe_lady = cond_fact.add_location("CafeLady", Condition::NoRequirements);
    let loc_hearth_ledge = cond_fact.add_location("HearthLedge", lantern_off);
    let loc_hearth_backdoor = cond_fact.add_location("HearthBackdoor", cond_or!(flippers, pacci_cane, rocs_cape));
    let loc_school_top = cond_fact.add_location("SchoolTop", pacci_cane);
    let loc_school_garden_left = cond_fact.add_location("SchoolGardenLeft", cond_and!(pacci_cane, flag_can_split4));
    let loc_school_garden_middle = cond_fact.add_location("SchoolGardenMiddle", cond_and!(pacci_cane, flag_can_split4));
    let loc_school_garden_right = cond_fact.add_location("SchoolGardenRight", cond_and!(pacci_cane, flag_can_split4));
    let loc_school_garden_heart_piece = cond_fact.add_location("SchoolGardenHeartPiece", cond_and!(pacci_cane, flag_can_split4));
    let loc_town_digging_top = cond_fact.add_location("TownDiggingTop", mole_mitts);
    let loc_town_digging_right = cond_fact.add_location("TownDiggingRight", mole_mitts);
    let loc_town_digging_left = cond_fact.add_location("TownDiggingLeft", mole_mitts);
    let loc_bakery_attic = cond_fact.add_location("BakeryAttic", cond_or!(pacci_cane, flippers, rocs_cape));
    let loc_stock_well_attic = cond_fact.add_location("StockWellAttic", cond_or!(pacci_cane, flippers, rocs_cape));
    let loc_simulation_chest = cond_fact.add_location("SimulationChest", flag_has_sword);
    let loc_rem_shoe_shop = cond_fact.add_location("RemShoeShop", wake_up_mushroom);
    // let loc_shop80_item = cond_fact.add_location("Shop80Item", todo!());
    // let loc_shop300_item = cond_fact.add_location("Shop300Item", cond_and!(wallet, todo!()));
    // let loc_shop600_item = cond_fact.add_location("Shop600Item", cond_and!(wallet, todo!()));
    let loc_shop_dogfood_item = cond_fact.add_location("ShopDogfoodItem", cond_or!(flippers, pacci_cane, rocs_cape));
    let loc_carlov_reward = cond_fact.add_location("CarlovReward", cond_or!(flippers, pacci_cane, rocs_cape));
    let loc_figurine_house_left = cond_fact.add_location("FigurineHouseLeft", carlov_medal);
    let loc_figurine_house_middle = cond_fact.add_location("FigurineHouseMiddle", carlov_medal);
    let loc_figurine_house_right = cond_fact.add_location("FigurineHouseRight", carlov_medal);
    let loc_figurine_house_heart_piece = cond_fact.add_location("FigurineHouseHeartPiece", carlov_medal);
    let loc_jullieta_book = cond_fact.add_location("JullietaBook", cond_and!(flag_has_bottle, cond_or!(flippers, pacci_cane, rocs_cape)));
    let loc_wright_attic_book = cond_fact.add_location("WrightAtticBook", cond_and!(power_bracelets, cond_or!(flippers, pacci_cane, rocs_cape), cond_or!(gust_jar, bomb_bag), flag_can_split2));
    let loc_fountain_big = cond_fact.add_location("FountainBig", cond_and!(flag_has_bottle, pacci_cane, flag_has_damage_source));
    let loc_fountain_small = cond_fact.add_location("FountainSmall", cond_and!(flag_has_bottle, cond_or!(flippers, rocs_cape)));
    let loc_fountain_heart_piece = cond_fact.add_location("FountainHeartPiece", cond_and!(flag_has_bottle, rocs_cape));
    let loc_library_minish = cond_fact.add_location("LibraryMinish", cond_and!(hyrulean_bestiary, picori_legend, mask_history, ocarina, pacci_cane));
    let loc_cucco_minigame = cond_fact.add_location("CuccoMinigame", cond_or!(rocs_cape, flippers));
    let loc_town_bell = cond_fact.add_location("TownBell", rocs_cape);
    let loc_flips_cave_big = cond_fact.add_location("FlipsCaveBig", cond_and!(ocarina, flag_has_damage_source, pacci_cane, cond_or!(flippers, cond_and!(hyrulean_bestiary, picori_legend, mask_history, grip_ring, cond_or!(gust_jar, rocs_cape)))));
    let loc_flips_cave_small = cond_fact.add_location("FlipsCaveSmall", cond_and!(flippers, ocarina, pacci_cane, lantern_off));
    let loc_tingle_trophy_item = cond_fact.add_location("TingleTrophyItem", cond_and!(flag_can_destroy_trees, pacci_cane, tingle_trophy));
    let loc_hills_keese_cave = cond_fact.add_location("HillsKeeseCave", bomb_bag);
    let loc_above_hp_hole = cond_fact.add_location("AboveHPHole", cond_and!(flag_access_hylia_north, cond_or!(pacci_cane, rocs_cape)));
    let loc_lon_lon_pot = cond_fact.add_location("LonLonPot", flag_access_east_field);
    let loc_lon_lon_cave = cond_fact.add_location("LonLonCave", cond_and!(flag_access_hylia_north, flag_can_split2));
    let loc_lon_lon_cave_secret = cond_fact.add_location("LonLonCaveSecret", cond_and!(loc_lon_lon_cave, bomb_bag, lantern_off));
    let loc_lon_lon_heart_piece = cond_fact.add_location("LonLonHeartPiece", cond_and!(flag_access_hylia_north, pegasus_boots));
    let loc_minish_rupee_fairy = cond_fact.add_location("MinishRupeeFairy", cond_and!(flag_access_east_field, pacci_cane));
    let loc_trilby_bomb_cave = cond_fact.add_location("TrilbyBombCave", cond_and!(flag_access_west_field, flag_can_split2, bomb_bag));
    let loc_trilby_mole_cave_left = cond_fact.add_location("TrilbyMoleCaveLeft", cond_and!(flag_access_west_field, mole_mitts));
    let loc_trilby_mole_cave_right = cond_fact.add_location("TrilbyMoleCaveRight", cond_and!(flag_access_west_field, mole_mitts));
    let loc_bottle_scrub = cond_fact.add_location("BottleScrub", cond_and!(shield, bomb_bag, flag_access_west_field));
    let loc_jabber_nut = cond_fact.add_location("JabberNut", flag_access_minish_woods);
    let loc_belari_bombs = cond_fact.add_location("BelariBombs", cond_and!(flag_access_minish_woods, cond_or!(flag_complete_deepwood, bomb_bag)));
    let loc_minish_middle_flipper_hole = cond_fact.add_location("MinishMiddleFlipperHole", cond_and!(flag_access_minish_woods, cond_or!(flag_complete_deepwood, bomb_bag), flippers));
    let loc_minish_right_flipper_hole = cond_fact.add_location("MinishRightFlipperHole", cond_and!(flag_access_minish_woods, cond_or!(flag_complete_deepwood, bomb_bag), flippers));
    let loc_minish_left_flipper_hole = cond_fact.add_location("MinishLeftFlipperHole", cond_and!(flag_access_minish_woods, cond_or!(flag_complete_deepwood, bomb_bag), flippers));
    let loc_minish_left_flipper_hole_heart_piece = cond_fact.add_location("MinishLeftFlipperHoleHeartPiece", cond_and!(flag_access_minish_woods, cond_or!(flag_complete_deepwood, bomb_bag), flippers));
    let loc_minish_like_like_digging_cave_left = cond_fact.add_location("MinishLikeLikeDiggingCaveLeft", cond_and!(flag_access_minish_woods, mole_mitts));
    let loc_minish_like_like_digging_cave_right = cond_fact.add_location("MinishLikeLikeDiggingCaveRight", cond_and!(flag_access_minish_woods, mole_mitts));
    let loc_minish_north_hole = cond_fact.add_location("MinishNorthHole", cond_and!(flag_access_hylia_south, flippers, pegasus_boots));
    let loc_minish_witch_hut = cond_fact.add_location("MinishWitchHut", cond_and!(flag_access_minish_woods, cond_or!(flippers, rocs_cape, cond_and!(pacci_cane, cond_or!(ocarina, lon_lon_key)))));
    let loc_minish_heart_piece_top = cond_fact.add_location("MinishHeartPieceTop", cond_and!(flag_access_minish_woods, cond_or!(flippers, rocs_cape, cond_and!(pacci_cane, cond_or!(ocarina, lon_lon_key)))));
    let loc_minish_heart_piece_bottom = cond_fact.add_location("MinishHeartPieceBottom", flag_access_minish_woods);
    let loc_minish_village_heart_piece = cond_fact.add_location("MinishVillageHeartPiece", flag_access_minish_woods);
    let loc_crenel_vine_hole = cond_fact.add_location("CrenelVineHole", flag_access_lower_crenel);
    let loc_crenel_minish_house = cond_fact.add_location("CrenelMinishHouse", flag_access_lower_crenel);
    let loc_crenel_cave_downstairs = cond_fact.add_location("CrenelCaveDownstairs", cond_and!(flag_access_crenel, bomb_bag));
    let loc_crenel_heart_cave_left = cond_fact.add_location("CrenelHeartCaveLeft", cond_and!(flag_access_lower_crenel, bomb_bag));
    let loc_crenel_heart_cave_right = cond_fact.add_location("CrenelHeartCaveRight", cond_and!(flag_access_lower_crenel, bomb_bag));
    let loc_crenel_heart_cave_heart_piece = cond_fact.add_location("CrenelHeartCaveHeartPiece", cond_and!(flag_access_lower_crenel, bomb_bag));
    let loc_crenel_fairy_heart_piece = cond_fact.add_location("CrenelFairyHeartPiece", cond_and!(flag_access_crenel, bomb_bag));
    let loc_crenel_grip_scrub = cond_fact.add_location("CrenelGripScrub", cond_and!(flag_access_crenel, shield, bomb_bag));
    let loc_grayblade_left = cond_fact.add_location("GraybladeLeft", cond_and!(flag_access_crenel, flag_can_split2, grip_ring));
    let loc_grayblade_right = cond_fact.add_location("GraybladeRight", cond_and!(flag_access_crenel, flag_can_split2, grip_ring));
    let loc_grayblade_heart_piece = cond_fact.add_location("GraybladeHeartPiece", cond_and!(flag_access_crenel, flag_can_split2, grip_ring));
    let loc_grayblade_scroll = cond_fact.add_location("GraybladeScroll", cond_and!(loc_grayblade_heart_piece, flag_has_sword));
    let loc_crenel_bomb_fairy = cond_fact.add_location("CrenelBombFairy", cond_and!(flag_access_crenel, bomb_bag, grip_ring));
    let loc_crenel_dig_cave_heart_piece = cond_fact.add_location("CrenelDigCaveHeartPiece", cond_and!(flag_access_crenel, grip_ring, mole_mitts));
    let loc_crenel_block_chest = cond_fact.add_location("CrenelBlockChest", cond_and!(flag_access_crenel, cond_or!(pacci_cane, cond_and!(grip_ring, cond_or!(rocs_cape, flag_has_light_bow, cond_and!(gust_jar, cond_or!(bomb_bag, flag_has_bow, flag_has_boomerang, flag_has_beam)))))));
    let loc_melari = cond_fact.add_location("Melari", flag_complete_co_f);
    let loc_wilds_south_cave = cond_fact.add_location("WildsSouthCave", cond_and!(flag_access_wilds, cond_or!(flippers, rocs_cape, flag_has_bow)));
    let loc_wilds_darknut_cave = cond_fact.add_location("WildsDarknutCave", cond_and!(flag_access_wilds, flag_has_sword));
    let loc_wilds_deku_cave_right = cond_fact.add_location("WildsDekuCaveRight", cond_and!(flag_access_wilds, flag_has_bow));
    let loc_wilds_mulldozer_hole = cond_fact.add_location("WildsMulldozerHole", cond_and!(flag_access_wilds, cond_or!(flippers, gust_jar)));
    let loc_wilds_digging_cave_left = cond_fact.add_location("WildsDiggingCaveLeft", cond_and!(flag_access_wilds, mole_mitts));
    let loc_wilds_digging_cave_right = cond_fact.add_location("WildsDiggingCaveRight", cond_and!(flag_access_wilds, mole_mitts));
    let loc_wilds_top_chest = cond_fact.add_location("WildsTopChest", cond_and!(flag_access_wilds, flag_has_bow));
    let loc_wilds_top_right_cave_heart_piece = cond_fact.add_location("WildsTopRightCaveHeartPiece", cond_and!(flag_access_wilds, flag_has_bow, cond_or!(flippers, rocs_cape)));
    let loc_swiftblade_the_first_heart_piece = cond_fact.add_location("SwiftbladeTheFirstHeartPiece", cond_and!(flag_access_wilds, cond_or!(flag_has_bow, rocs_cape, flippers)));
    // let loc_swiftblade_the_first_scroll = cond_fact.add_location("SwiftbladeTheFirstScroll", cond_and!(flag_has_sword, loc_swiftblade_the_first_heart_piece, flag_got_scrolls));
    let loc_ruins_bomb_cave = cond_fact.add_location("RuinsBombCave", cond_and!(flag_access_wilds, cond_or!(flag_has_bow, rocs_cape, flippers), kinstone_x_yellow_totem_prong, bomb_bag));
    let loc_ruins_minish_home = cond_fact.add_location("RuinsMinishHome", cond_and!(flag_access_wilds, cond_or!(flag_has_bow, rocs_cape, flippers), kinstone_x_yellow_totem_prong, flag_has_sword));
    let loc_ruins_minish_cave_heart_piece = cond_fact.add_location("RuinsMinishCaveHeartPiece", cond_and!(flag_access_wilds, cond_or!(flag_has_bow, rocs_cape, flippers), kinstone_x_yellow_totem_prong, flag_has_sword));
    let loc_ruins_armos_kill_left = cond_fact.add_location("RuinsArmosKillLeft", cond_and!(flag_access_wilds, cond_or!(flag_has_bow, rocs_cape, flippers), kinstone_x_yellow_totem_prong, flag_has_sword));
    let loc_ruins_armos_kill_right = cond_fact.add_location("RuinsArmosKillRight", cond_and!(flag_access_wilds, cond_or!(flag_has_bow, rocs_cape, flippers), kinstone_x_yellow_totem_prong, flag_has_sword));
    let loc_stockwell_dog = cond_fact.add_location("StockwellDog", cond_and!(flag_access_hylia_north, dog_food_bottle));
    let loc_hylia_north_minish_hole = cond_fact.add_location("HyliaNorthMinishHole", cond_and!(flag_access_hylia_north, flippers, pegasus_boots));
    let loc_hylia_mayor_cabin = cond_fact.add_location("HyliaMayorCabin", cond_and!(flag_access_hylia_south, pegasus_boots, power_bracelets, cond_or!(gust_jar, flippers)));
    let loc_witch_digging_cave = cond_fact.add_location("WitchDiggingCave", cond_and!(flag_access_hylia_south, mole_mitts));
    let loc_hylia_sunken_heart_piece = cond_fact.add_location("HyliaSunkenHeartPiece", cond_and!(flag_access_hylia_north, flippers));
    let loc_hylia_bottom_heart_piece = cond_fact.add_location("HyliaBottomHeartPiece", cond_and!(flag_access_hylia_north, cond_or!(flippers, rocs_cape)));
    let loc_waveblade_heart_piece = cond_fact.add_location("WavebladeHeartPiece", cond_and!(flag_access_hylia_north, cond_or!(flippers, rocs_cape)));
    // let loc_waveblade_scroll = cond_fact.add_location("WavebladeScroll", cond_and!(todo!(), flag_has_sword, loc_waveblade_heart_piece));
    let loc_hylia_cape_cave_top_right = cond_fact.add_location("HyliaCapeCaveTopRight", flag_access_treasure_cave);
    let loc_hylia_cape_cave_bottom_left = cond_fact.add_location("HyliaCapeCaveBottomLeft", flag_access_treasure_cave);
    let loc_hylia_cape_cave_top_left = cond_fact.add_location("HyliaCapeCaveTopLeft", flag_access_treasure_cave);
    let loc_hylia_cape_cave_top_middle = cond_fact.add_location("HyliaCapeCaveTopMiddle", flag_access_treasure_cave);
    let loc_hylia_cape_cave_entrance = cond_fact.add_location("HyliaCapeCaveEntrance", flag_access_treasure_cave);
    let loc_hylia_cape_cave_bottom_right = cond_fact.add_location("HyliaCapeCaveBottomRight", flag_access_treasure_cave);
    let loc_hylia_cape_cave_bottom_middle = cond_fact.add_location("HyliaCapeCaveBottomMiddle", flag_access_treasure_cave);
    let loc_hylia_post_cape_cave_heart_piece = cond_fact.add_location("HyliaPostCapeCaveHeartPiece", flag_access_treasure_cave);
    let loc_hylia_pre_cape_cave_heart_piece = cond_fact.add_location("HyliaPreCapeCaveHeartPiece", cond_and!(flag_access_hylia_north, rocs_cape));
    let loc_arrow_fairy = cond_fact.add_location("ArrowFairy", cond_and!(flag_can_split3, bomb_bag));
    let loc_dampe_key = cond_fact.add_location("DampeKey", flag_access_valley);
    let loc_royal_valley_grave_heart_piece = cond_fact.add_location("RoyalValleyGraveHeartPiece", cond_and!(flag_access_valley, graveyard_key, pegasus_boots));
    let loc_royal_valley_lost_woods_chest = cond_fact.add_location("RoyalValleyLostWoodsChest", flag_access_valley);
    let loc_crypt_gibdo_left = cond_fact.add_location("CryptGibdoLeft:Crypt", flag_access_crypt);
    let loc_king_gift = cond_fact.add_location("KingGift:`ELEMENT_DUNGEON`", cond_and!(flag_access_valley, graveyard_key, flag_can_split3, pegasus_boots, small_key_rc_set_3));
    let loc_falls_behind_wall = cond_fact.add_location("FallsBehindWall", cond_and!(flag_access_falls_north, bomb_bag));
    let loc_falls_cliff = cond_fact.add_location("FallsCliff", cond_and!(flag_access_falls_north, bomb_bag, flag_can_split3));
    let loc_falls_top_cave_bomb = cond_fact.add_location("FallsTopCaveBomb", cond_and!(flag_access_falls_north, grip_ring, bomb_bag));
    let loc_falls_top_cave_free = cond_fact.add_location("FallsTopCaveFree", cond_and!(flag_access_falls_north, grip_ring));
    let loc_falls_upper_heart_piece = cond_fact.add_location("FallsUpperHeartPiece", cond_and!(bomb_bag, cond_or!(rocs_cape, flippers)));
    let loc_falls_lower_cave_left = cond_fact.add_location("FallsLowerCaveLeft", cond_and!(flag_access_falls_south, cond_or!(rocs_cape, flippers), mole_mitts));
    let loc_falls_lower_cave_right = cond_fact.add_location("FallsLowerCaveRight", cond_and!(flag_access_falls_south, cond_or!(rocs_cape, flippers), mole_mitts));
    let loc_falls_lower_heart_piece = cond_fact.add_location("FallsLowerHeartPiece", flag_access_falls_south);
    let loc_clouds_free_chest = cond_fact.add_location("CloudsFreeChest", flag_access_clouds);
    let loc_clouds_north_kill = cond_fact.add_location("CloudsNorthKill", cond_and!(flag_access_clouds, cond_or!(rocs_cape, mole_mitts)));
    let loc_clouds_south_kill = cond_fact.add_location("CloudsSouthKill", cond_and!(flag_access_clouds, cond_or!(rocs_cape, mole_mitts)));
    let loc_clouds_south_middle = cond_fact.add_location("CloudsSouthMiddle", cond_and!(flag_access_clouds, cond_or!(rocs_cape, mole_mitts)));
    let loc_clouds_west_bottom = cond_fact.add_location("CloudsWestBottom", cond_and!(flag_access_clouds, cond_or!(rocs_cape, mole_mitts)));
    let loc_clouds_west_left = cond_fact.add_location("CloudsWestLeft", cond_and!(flag_access_clouds, mole_mitts));
    let loc_clouds_west_right = cond_fact.add_location("CloudsWestRight", cond_and!(flag_access_clouds, mole_mitts));
    let loc_clouds_south_left = cond_fact.add_location("CloudsSouthLeft", cond_and!(flag_access_clouds, mole_mitts));
    let loc_clouds_south_right = cond_fact.add_location("CloudsSouthRight", cond_and!(flag_access_clouds, cond_or!(rocs_cape, mole_mitts)));
    let loc_gregal_two = cond_fact.add_location("GregalTwo", cond_and!(flag_access_upper_clouds, gust_jar));
    let loc_tower_right_bed = cond_fact.add_location("TowerRightBed", flag_access_upper_clouds);
    let loc_tower_middle_bed = cond_fact.add_location("TowerMiddleBed", flag_access_upper_clouds);
    let loc_tower_left_bed = cond_fact.add_location("TowerLeftBed", flag_access_upper_clouds);
    let loc_tower_top_left = cond_fact.add_location("TowerTopLeft", flag_access_upper_clouds);
    let loc_tower_top_right = cond_fact.add_location("TowerTopRight", flag_access_upper_clouds);
    let loc_deepwood_wiggler = cond_fact.add_location("DeepwoodWiggler:Deepwood", cond_and!(flag_deepwood_access, flag_has_sword, cond_or!(small_key_dws_set_4, cond_and!(small_key_dws_set_2, lantern_off), cond_and!(small_key_dws_set, lantern_off, gust_jar))));
    let loc_deepwood_post_wiggler_heart_piece = cond_fact.add_location("DeepwoodPostWigglerHeartPiece:Deepwood", cond_and!(flag_deepwood_access, cond_or!(gust_jar, lantern_off), cond_or!(small_key_dws_set_4, cond_and!(small_key_dws_set_2, lantern_off), cond_and!(small_key_dws_set, lantern_off, gust_jar))));
    let loc_deepwood_pre_wiggler_left = cond_fact.add_location("DeepwoodPreWigglerLeft:Deepwood", cond_and!(flag_deepwood_access, cond_or!(cond_and!(gust_jar, small_key_dws_set), cond_and!(bomb_bag, small_key_dws_set_2))));
    let loc_deepwood_pre_wiggler_right = cond_fact.add_location("DeepwoodPreWigglerRight:Deepwood", cond_and!(flag_deepwood_access, cond_or!(cond_and!(gust_jar, small_key_dws_set), cond_and!(bomb_bag, small_key_dws_set_2))));
    let loc_deepwood_pre_wiggler_heart_piece = cond_fact.add_location("DeepwoodPreWigglerHeartPiece:Deepwood", cond_and!(flag_deepwood_access, cond_or!(cond_and!(gust_jar, small_key_dws_set), cond_and!(bomb_bag, small_key_dws_set_2))));
    let loc_deepwood_pre_compass = cond_fact.add_location("DeepwoodPreCompass:Deepwood", cond_and!(flag_deepwood_access, cond_or!(cond_and!(gust_jar, small_key_dws_set), cond_and!(bomb_bag, small_key_dws_set_2))));
    let loc_deepwood_pre_barrel = cond_fact.add_location("DeepwoodPreBarrel:Deepwood", cond_and!(flag_deepwood_access, cond_or!(gust_jar, bomb_bag), small_key_dws_set));
    let loc_deepwood_upstairs_chest = cond_fact.add_location("DeepwoodUpstairsChest:Deepwood", cond_and!(flag_deepwood_access, cond_or!(gust_jar, lantern_off)));
    let loc_deepwood_boss_item = cond_fact.add_location("DeepwoodBossItem:Deepwood", flag_complete_deepwood);
    let loc_co_f_heart_piece = cond_fact.add_location("CoFHeartPiece:FlameCave", cond_and!(flag_co_f_access, bomb_bag, flag_has_sword, small_key_cof_set));
    let loc_co_f_chu_pit = cond_fact.add_location("CoFChuPit:FlameCave", cond_and!(flag_co_f_access, small_key_cof_set, flag_has_sword));
    let loc_co_f_pill_bugs_hole_chest = cond_fact.add_location("CoFPillBugsHoleChest:FlameCave", flag_co_f_access);
    let loc_co_f_southeast_small = cond_fact.add_location("CoFSoutheastSmall:FlameCave", flag_co_f_access);
    let loc_co_f_basement_top = cond_fact.add_location("CoFBasementTop:FlameCave", cond_and!(flag_co_f_access, pacci_cane, flag_has_sword, small_key_cof_set_2));
    let loc_co_f_basement_bottom = cond_fact.add_location("CoFBasementBottom:FlameCave", cond_and!(flag_co_f_access, pacci_cane, flag_has_sword, small_key_cof_set_2));
    let loc_co_f_blades = cond_fact.add_location("CoFBlades:FlameCave", cond_and!(flag_co_f_access, pacci_cane, flag_has_sword, small_key_cof_set_2));
    let loc_co_f_basement_lava_left = cond_fact.add_location("CoFBasementLavaLeft:FlameCave", cond_and!(flag_co_f_access, pacci_cane, flag_has_sword, small_key_cof_set_2));
    let loc_co_f_basement_lava_right = cond_fact.add_location("CoFBasementLavaRight:FlameCave", cond_and!(flag_co_f_access, pacci_cane, flag_has_sword, small_key_cof_set_2));
    let loc_co_f_boss_item = cond_fact.add_location("CoFBossItem:FlameCave", flag_complete_co_f);
    let loc_fortress_entrance = cond_fact.add_location("FortressEntrance:Fortress", cond_and!(flag_access_fortress, mole_mitts));
    let loc_fortress_heart_piece = cond_fact.add_location("FortressHeartPiece:Fortress", cond_and!(flag_access_fortress, flag_can_split2));
    let loc_fortress_outside_f2_left = cond_fact.add_location("FortressOutsideF2Left:Fortress", cond_and!(flag_access_fortress, mole_mitts, flag_has_bow));
    let loc_fortress_outside_f2_middle = cond_fact.add_location("FortressOutsideF2Middle:Fortress", cond_and!(flag_access_fortress, mole_mitts));
    let loc_fortress_outside_f2_right = cond_fact.add_location("FortressOutsideF2Right:Fortress", cond_and!(flag_access_fortress, mole_mitts));
    let loc_fortress_outside_f3_left = cond_fact.add_location("FortressOutsideF3Left:Fortress", cond_and!(flag_access_fortress, mole_mitts, flag_has_bow));
    let loc_fortress_outside_f3_right = cond_fact.add_location("FortressOutsideF3Right:Fortress", cond_and!(flag_access_fortress, mole_mitts));
    let loc_fortress_outside_bomb_wall_big_chest = cond_fact.add_location("FortressOutsideBombWallBigChest:Fortress", cond_and!(flag_access_fortress, bomb_bag, flag_has_bow, flag_can_split2, small_key_fow_set_4));
    let loc_fortress_outside_bomb_wall_small_chest = cond_fact.add_location("FortressOutsideBombWallSmallChest:Fortress", cond_and!(loc_fortress_outside_bomb_wall_big_chest, mole_mitts));
    let loc_fortress_skull_room_left = cond_fact.add_location("FortressSkullRoomLeft:Fortress", flag_access_fortress);
    let loc_fortress_skull_room_right = cond_fact.add_location("FortressSkullRoomRight:Fortress", flag_access_fortress);
    let loc_fortress_wizrobes = cond_fact.add_location("FortressWizrobes:Fortress", cond_and!(flag_access_fortress, mole_mitts));
    let loc_fortress_boss_item = cond_fact.add_location("FortressBossItem:Fortress", flag_complete_fortress);
    let loc_fortress_prize = cond_fact.add_location("FortressPrize:`ELEMENT_DUNGEON`", flag_complete_fortress);
    let loc_droplets_east_first = cond_fact.add_location("DropletsEastFirst:Droplets", cond_and!(flag_access_droplets, big_key_tod_set, cond_or!(lantern_off, flag_droplets_east_lever)));
    let loc_droplets_ice_maze = cond_fact.add_location("DropletsIceMaze:Droplets", cond_and!(flag_access_droplets, big_key_tod_set, cond_or!(lantern_off, flag_droplets_east_lever)));
    let loc_droplets_blu_chu = cond_fact.add_location("DropletsBluChu:Droplets", cond_and!(flag_access_droplets, big_key_tod_set, cond_or!(lantern_off, flag_droplets_east_lever), gust_jar, small_key_tod_set_4));
    let loc_droplets_frozen_ice_plain = cond_fact.add_location("DropletsFrozenIcePlain:Droplets", cond_and!(flag_access_droplets, big_key_tod_set, cond_or!(flag_droplets_bottom_jump, cond_and!(small_key_tod_set_4, gust_jar, flippers, lantern_off))));
    let loc_droplets_free_ice_plain = cond_fact.add_location("DropletsFreeIcePlain:Droplets", cond_and!(flag_access_droplets, big_key_tod_set, cond_or!(flag_droplets_bottom_jump, cond_and!(small_key_tod_set_4, gust_jar, flippers))));
    let loc_droplets_dark_maze_right = cond_fact.add_location("DropletsDarkMazeRight:Droplets", cond_and!(flag_access_droplets, big_key_tod_set, lantern_off, flag_has_damage_source));
    let loc_droplets_dark_maze_left = cond_fact.add_location("DropletsDarkMazeLeft:Droplets", cond_and!(flag_access_droplets, big_key_tod_set, lantern_off, flag_has_damage_source));
    let loc_droplets_dark_maze_middle = cond_fact.add_location("DropletsDarkMazeMiddle:Droplets", cond_and!(flag_access_droplets, big_key_tod_set, lantern_off, flag_has_damage_source));
    let loc_droplets_post_twin_frozen = cond_fact.add_location("DropletsPostTwinFrozen:Droplets", cond_and!(flag_access_droplets, big_key_tod_set, gust_jar, lantern_off, cond_or!(flag_droplets_bottom_jump, cond_and!(small_key_tod_set_4, flippers))));
    let loc_droplets_preview_frozen = cond_fact.add_location("DropletsPreviewFrozen:Droplets", cond_and!(flag_access_droplets, big_key_tod_set, lantern_off));
    let loc_droplets_boss_item = cond_fact.add_location("DropletsBossItem:Droplets", flag_complete_droplets);
    let loc_palace_wizrobe_kill = cond_fact.add_location("PalaceWizrobeKill:Palace", cond_and!(flag_access_palace, cond_or!(rocs_cape, bomb_bag, flag_has_boomerang)));
    let loc_palace_first_grate = cond_fact.add_location("PalaceFirstGrate:Palace", cond_and!(flag_access_palace, rocs_cape));
    let loc_palace_wide_gap = cond_fact.add_location("PalaceWideGap:Palace", cond_and!(flag_access_palace, rocs_cape, pacci_cane));
    let loc_palace_twin_wizrobes = cond_fact.add_location("PalaceTwinWizrobes:Palace", cond_and!(flag_access_palace, rocs_cape, pacci_cane, lantern_off));
    let loc_palace_heart_piece = cond_fact.add_location("PalaceHeartPiece:Palace", cond_and!(flag_access_palace, rocs_cape, pacci_cane, lantern_off));
    let loc_palace_switch_hit = cond_fact.add_location("PalaceSwitchHit:Palace", cond_and!(flag_access_palace, rocs_cape, pacci_cane, lantern_off));
    let loc_palace_block_maze = cond_fact.add_location("PalaceBlockMaze:Palace", cond_and!(flag_access_palace, rocs_cape, pacci_cane, lantern_off));
    let loc_palace_detour = cond_fact.add_location("PalaceDetour:Palace", cond_and!(flag_access_palace, rocs_cape, pacci_cane, lantern_off));
    let loc_palace_boss_item = cond_fact.add_location("PalaceBossItem:Palace", flag_complete_palace);
    let loc_castle_king = cond_fact.add_location("CastleKing:DHC", cond_and!(flag_dhc_access, flag_can_split4, bomb_bag));
    
    let junk = cond_fact.add_item("Junk", ItemCategory::Minor);
    allocate(&cond_fact, cond_fact.get_item(junk).unwrap());
}