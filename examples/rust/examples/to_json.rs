use planus::{ReadAsRoot, ToOwned};
use planus_example::monster_generated::my_game::sample::*;

pub fn main() {
    let path = std::env::args().nth(1).unwrap();
    let data = std::fs::read(path).unwrap();
    let monster = MonsterRef::read_as_root(&data).unwrap().to_owned().unwrap();

    println!("{}", serde_json::to_string_pretty(&monster).unwrap());
}
