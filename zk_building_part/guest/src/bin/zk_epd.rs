use risc0_zkvm::guest::{abort, env};
use zk_building_part_guest::{ConcreteEpd, ConcreteMixture};

fn main() {
    // read the input
    let mixture: ConcreteMixture = env::read();
    let factory_wien_gwp = 3.9f64;
    let factory_eggendorf_gwp = 3.6f64;
    let cement_gwp = 1.20f64;
    let additive_gwp = 0.15f64;
    let gravel_gwp = 0.009f64;
    let water_gwp = 0.005f64;

    let factory_gwp = if mixture.factory == "Wien" {
        factory_wien_gwp
    } else if mixture.factory == "Eggendorf" {
        factory_eggendorf_gwp
    } else { abort("Unknown factory!") };

    if mixture.cement + mixture.water + mixture.additives + mixture.gravel != 1000.0 {
        abort("Concrete mixture does not sum up to 1000!")
    }

    let gwp = mixture.cement * cement_gwp +
        mixture.gravel * gravel_gwp +
        mixture.water * water_gwp +
        mixture.additives * additive_gwp +
        factory_gwp;

    let epd = ConcreteEpd {
        description: mixture.description,
        factory: mixture.factory,
        a13_gwp: gwp,
    };
    // write public output to the journal
    env::commit(&epd);
}
