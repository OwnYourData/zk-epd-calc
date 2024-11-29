/*
 * Copyright (c) 2024 Thomas Preindl
 * MIT License (see LICENSE or https://mit-license.org)
 */

use risc0_zkvm::guest::env;
use zk_building_part_guest::{BuildingDefinition, BuildingEpd};

fn main() {
    let BuildingDefinition {
        date,
        building,
        building_parts,
        site_energy,
    } = env::read();

    let a14_gwp: u32 = building_parts.iter().map(|bp_epd| bp_epd.a14_gwp).sum();

    let a5_gwp: u32 = site_energy
        .iter()
        .map(|energy| energy.gwp * energy.amount)
        .sum();

    let a15_gwp = a14_gwp + a5_gwp;

    let epd = BuildingEpd {
        date,
        building,
        a15_gwp,
    };

    env::commit(&epd)
}
