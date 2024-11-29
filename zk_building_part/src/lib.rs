/*
 * Copyright (c) 2024 Thomas Preindl
 * MIT License (see LICENSE or https://mit-license.org)
 */

mod guest_methods;

use zk_building_part_guest::{
    BuildingDefinition, BuildingEpd, BuildingPartDefinition, BuildingPartEPD, ConcreteEpd,
    ConcreteMixture, ZkBuildingEpd, ZkBuildingPartEPD, ZkConcreteEpd,
};
use zk_epdcalc::{new_config_factory, ConfigFactory};

pub fn builder() -> Vec<(&'static str, Box<dyn ConfigFactory + Send + Sync>)> {
    vec![
        (
            "Concrete",
            new_config_factory::<ConcreteMixture, ConcreteEpd, ZkConcreteEpd>(
                guest_methods::ZK_EPD_ELF,
                &guest_methods::ZK_EPD_ID,
            ),
        ),
        (
            "BuildingPart",
            new_config_factory::<BuildingPartDefinition, BuildingPartEPD, ZkBuildingPartEPD>(
                guest_methods::ZK_BUILDING_PART_ELF,
                &guest_methods::ZK_BUILDING_PART_ID,
            ),
        ),
        (
            "Building",
            new_config_factory::<BuildingDefinition, BuildingEpd, ZkBuildingEpd>(
                guest_methods::ZK_BUILDING_ELF,
                &guest_methods::ZK_BUILDING_ID,
            ),
        ),
    ]
}
