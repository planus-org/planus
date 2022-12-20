use planus::ReadAsRoot;
use planus_example::monster_generated::my_game::sample::*;

pub fn main() {
    let path = std::env::args().nth(1).unwrap();
    let data = std::fs::read(path).unwrap();
    let monster = MonsterRef::read_as_root(&data).unwrap();
    println!("{monster:#?}");
}
