use planus::{Builder, ReadAsRoot, WriteAsOffset};
use planus_example::monster_generated::my_game::sample::*;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Usage: api_example (output file)");
    let mut builder = Builder::new();

    // Create an owned version of the monster to serialize
    let monster = Monster {
        pos: Some(Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        }),
        mana: 150,
        hp: 80,
        name: Some("Orc".to_string()),
        inventory: Some(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
        color: Color::Red,
        weapons: Some(vec![
            Weapon {
                name: Some("Sword".to_string()),
                damage: 3,
            },
            Weapon {
                name: Some("Axe".to_string()),
                damage: 5,
            },
        ]),
        equipped: Some(Equipment::Weapon(Box::new(Weapon {
            name: Some("Sword".to_string()),
            damage: 3,
        }))),
        path: Some(vec![
            Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            Vec3 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
        ]),
    };

    // We can finish using the monster directly
    let _finished_data = builder.finish(&monster, None);
    builder.clear();

    // Or using an offset to it
    let monster_offset = monster.prepare(&mut builder);
    let _finished_data = builder.finish(monster_offset, None);

    // To avoid using the rust heap for objects and additionally allow sharing of data, use `create` methods instead:
    let weapons = [
        Weapon::create(&mut builder, "Sword", 3),
        Weapon::create(&mut builder, "Axe", 5),
    ];
    let equipped = Equipment::create_weapon(&mut builder, weapons[1]);
    let monster = Monster::create(
        &mut builder,
        Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        150,
        80,
        "Orc",
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        Color::Red,
        weapons,
        equipped,
        [
            Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            Vec3 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
        ],
    );

    let finished_data = builder.finish(monster, None);

    std::fs::write(path, finished_data).unwrap();

    // We can decode the data using planus::ReadAsRoot
    let monster_ref: MonsterRef<'_> = MonsterRef::read_as_root(finished_data).unwrap();
    print_equipment(monster_ref).unwrap();
}

fn print_equipment(monster: MonsterRef<'_>) -> Result<(), planus::Error> {
    // All accessors on tables return Result<_, planus::Error>
    // If the field is optional, then an Result<Option<_>, planus::Error>.
    if let Some(equipped) = monster.equipped()? {
        // Unions translate to rust enums with data
        match equipped {
            EquipmentRef::Weapon(weapon) => {
                // All generated types implement Debug
                println!("{:?}", weapon);
            }
        }
    }
    Ok(())
}
