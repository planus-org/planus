use planus::Builder;
use planus_example::monster_generated::my_game::sample::*;

pub fn main() {
    let input_path = std::env::args()
        .nth(1)
        .expect("Usage: from_json (input file) (output file)");
    let output_path = std::env::args()
        .nth(2)
        .expect("Usage: from_json (input file) (output file)");
    let input_data = std::fs::read_to_string(input_path).unwrap();

    let monster: Monster = serde_json::from_str(&input_data).unwrap();
    let mut builder = Builder::new();
    let output_data = builder.finish(monster, None);

    std::fs::write(output_path, output_data).unwrap();
}
