use std::{io::Error, path::Path, collections::{HashMap, HashSet}, hash::Hash};
mod logic_parse;
mod header;
mod logic;

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

#[derive(Debug, Clone, StructOpt)]
struct Args {
    logic_path: String,
    #[structopt(long)]
    sadistic: bool,
    #[structopt(long)]
    match_category: bool,
    #[structopt(long)]
    temperature: Option<u32>,
}

use logic::{Logic, Allocator, Item, LogicLoader};
use logic_parse::parse_logic;
use structopt::StructOpt;
fn main() {
    // Rom::new(Path::new("foo.rom"));
    let args = Args::from_args();
    let f = std::fs::File::open(&args.logic_path).unwrap();
    let (logic, item_pool_ids) = LogicLoader::from_reader(f);
    let item_pool: Vec<_> = item_pool_ids.into_iter().map(|id| Item::new(logic.get_item(id).unwrap())).collect();
    let mut allocator = Allocator::new(&logic, item_pool, args.sadistic, args.match_category, args.temperature.unwrap_or(5));
    allocator.allocate(&mut rand::thread_rng());
    // parse_logic(include_str!("default.logic.txt"));
    

}
