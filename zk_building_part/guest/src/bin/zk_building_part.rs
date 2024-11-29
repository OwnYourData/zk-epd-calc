use risc0_zkvm::guest::env;
use zk_building_part_guest::{BuildingPartDefinition, BuildingPartEPD};

fn main() {
    // read the input
    let definition: BuildingPartDefinition = env::read();

    let a14_gwp= definition.used_material.iter().map(|m| {m.a13_gwp * m.amount }).sum();

    let epd = BuildingPartEPD{
        date: definition.date,
        building: definition.building,
        building_part_id: definition.building_part_id,
        a14_gwp,
    };
    // write public output to the journal
    env::commit(&epd);
}