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

    // To avoid using the rust heap for objects and additionally allow sharing of data, use the builder API instead:
    let weapons = [
        Weapon::builder()
            .name("Sword")
            .damage(3)
            .finish(&mut builder),
        Weapon::builder().name("Axe").damage(5).finish(&mut builder),
    ];
    let equipped = Equipment::builder().weapon(weapons[1]).finish(&mut builder);
    let monster = Monster::builder()
        .pos(Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        })
        .mana(150)
        .hp(80)
        .name("Orc")
        .inventory([0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
        .color(Color::Red)
        .weapons(weapons)
        .equipped(equipped)
        .path([
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
        ])
        .finish(&mut builder);

    let finished_data = builder.finish(monster, None);

    std::fs::write(path, finished_data).unwrap();

    // We can decode the data using planus::ReadAsRoot
    let monster_ref = MonsterRef::read_as_root(finished_data).unwrap();
    print_equipment(monster_ref).unwrap();

    // And we can get an owned version by using TryInto
    let _monster: Monster = monster_ref.try_into().unwrap();
}

fn print_equipment(monster: MonsterRef<'_>) -> Result<(), planus::Error> {
    // All accessors on tables return Result<_, planus::Error>
    // If the field is optional, then an Result<Option<_>, planus::Error>.
    if let Some(equipped) = monster.equipped()? {
        // Unions translate to rust enums with data
        match equipped {
            EquipmentRef::Weapon(weapon) => {
                // All generated types implement Debug
                println!("{weapon:?}");
            }
        }
    }
    Ok(())
}
