#[cfg(target_arch = "x86_64")]
use anyhow::{anyhow, Result};
#[cfg(target_arch = "x86_64")]
use itertools::Itertools;
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "x86_64")]
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use zk_epdcalc_core::VerifiedEpd;

#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(
    target_arch = "x86_64",
    serde(try_from = "CreateBuildingPartRequestDTO")
)]
pub struct BuildingPartDefinition {
    pub date: Box<str>,
    pub building: Box<str>,
    pub building_part_id: Box<str>,
    pub used_material: Vec<MaterialUse>,
}

#[cfg(target_arch = "x86_64")]
impl TryFrom<CreateBuildingPartRequestDTO> for BuildingPartDefinition {
    type Error = anyhow::Error;

    fn try_from(request: CreateBuildingPartRequestDTO) -> Result<Self, Self::Error> {
        let used_material_dto = &request.definition.used_material;
        let used_material: Result<Vec<MaterialUse>> = used_material_dto
            .iter()
            .map(|material_use: &MaterialUseDto| {
                let amount = material_use.amount;
                let did = &*material_use.concrete_dpp_did;
                let mapping = request.get_did_mapping(did)?;
                let vc = mapping.get_vc()?;
                let a13_gwp = vc.credential_subject.epd.epd.a13_gwp;
                if a13_gwp > u32::MAX as f64 || a13_gwp < 0f64 {
                    Err(anyhow!("a13_gwp out of range: {a13_gwp}"))?;
                }
                let a13_gwp: u32 = a13_gwp as u32;
                Ok(MaterialUse { amount, a13_gwp }) as Result<MaterialUse, Self::Error>
            })
            .collect();

        Ok(Self {
            used_material: used_material?,
            date: request.definition.date,
            building: request.definition.building,
            building_part_id: request.definition.building_part_id,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MaterialUse {
    pub amount: u32,
    pub a13_gwp: u32,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConcreteMixture {
    pub description: String,
    pub cement: f64,
    pub gravel: f64,
    pub water: f64,
    pub additives: f64,
    pub material: String,
    pub factory: String,
}

#[cfg(target_arch = "x86_64")]
#[derive(Deserialize, Debug)]
pub struct CreateBuildingPartRequestDTO {
    pub definition: BuildingPartDefinitionDto,
    pub dpps: Vec<DidMappingDto<ConcreteMixtureDppDto>>,
}

#[cfg(target_arch = "x86_64")]
impl CreateBuildingPartRequestDTO {
    fn get_did_mapping(&self, did: &str) -> Result<&DidMappingDto<ConcreteMixtureDppDto>> {
        self.dpps
            .iter()
            .find(|did_mapping| did == did_mapping.did)
            .ok_or(anyhow!("Did document mapping not provided for {did}!"))
    }
}

#[cfg(target_arch = "x86_64")]
#[derive(Deserialize, Debug)]
pub struct DidMappingDto<T> {
    pub did: String,
    pub dpp_vp: VerifiablePresentationDTO<T>,
}

#[cfg(target_arch = "x86_64")]
impl<T> DidMappingDto<T> {
    fn get_vc(&self) -> Result<&VerifiableCredentialDTO<T>> {
        self.dpp_vp.vcs.first().ok_or(anyhow!(
            "Missing Verifiable Credential in Verifiable Presentation ({})!",
            self.did
        ))
    }
}

#[cfg(target_arch = "x86_64")]
#[derive(Deserialize, Debug)]
pub struct VerifiablePresentationDTO<Subject> {
    #[serde(rename = "verifiableCredential")]
    pub vcs: Vec<VerifiableCredentialDTO<Subject>>,
}

#[cfg(target_arch = "x86_64")]
#[derive(Deserialize, Debug)]
pub struct VerifiableCredentialDTO<Subject> {
    #[serde(rename = "credentialSubject")]
    pub credential_subject: Subject,
}

#[cfg(target_arch = "x86_64")]
#[derive(Deserialize, Debug)]
pub struct BuildingPartDefinitionDto {
    pub date: Box<str>,
    pub building: Box<str>,
    #[serde(rename = "buildingPartID")]
    pub building_part_id: Box<str>,
    #[serde(rename = "usedMaterial")]
    pub used_material: Vec<MaterialUseDto>,
}

#[cfg(target_arch = "x86_64")]
#[derive(Deserialize, Debug)]
pub struct MaterialUseDto {
    pub amount: u32,
    #[serde(rename = "concreteDppDid")]
    pub concrete_dpp_did: String,
}

#[cfg(target_arch = "x86_64")]
#[derive(Deserialize, Debug)]
pub struct ConcreteMixtureDppDto {
    pub id: String,
    pub epd: ZkConcreteEpd,
    pub date: String,
    pub volume: u32,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConcreteEpd {
    pub description: String,
    pub factory: String,
    #[serde(rename = "A13_gwp")]
    pub a13_gwp: f64,
}

impl PartialEq for ConcreteEpd {
    fn eq(&self, other: &Self) -> bool {
        self.a13_gwp == other.a13_gwp
            && self.factory == other.factory
            && self.description == other.description
    }
}

impl Eq for ConcreteEpd {}

#[cfg(target_arch = "x86_64")]
#[derive(Deserialize, Serialize, Clone)]
pub struct ZkConcreteEpd {
    #[serde(flatten)]
    pub epd: ConcreteEpd,
    pub zkp: Box<str>,
}

#[cfg(target_arch = "x86_64")]
impl Debug for ZkConcreteEpd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.epd.fmt(f)
    }
}

#[cfg(target_arch = "x86_64")]
impl VerifiedEpd<ConcreteEpd> for ZkConcreteEpd {
    fn get_zkp(&self) -> &str {
        &self.zkp
    }

    fn get_epd(&self) -> &ConcreteEpd {
        &self.epd
    }

    fn from_result(epd: ConcreteEpd, zkp: Box<str>) -> Self {
        Self { epd, zkp }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct BuildingPartEPD {
    pub date: Box<str>,
    pub building: Box<str>,
    pub building_part_id: Box<str>,
    pub a14_gwp: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ZkBuildingPartEPD {
    #[serde(flatten)]
    pub epd: Arc<BuildingPartEPD>,
    pub zkp: Box<str>,
}

impl Debug for ZkBuildingPartEPD {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.epd.fmt(f)
    }
}

impl VerifiedEpd<BuildingPartEPD> for ZkBuildingPartEPD {
    fn get_zkp(&self) -> &str {
        &self.zkp
    }

    fn get_epd(&self) -> &BuildingPartEPD {
        &self.epd
    }

    fn from_result(epd: BuildingPartEPD, zkp: Box<str>) -> Self {
        ZkBuildingPartEPD {
            zkp,
            epd: Arc::new(epd),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ConstructionSiteEnergy {
    pub amount: u32,
    pub gwp: u32,
}

#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(
    target_arch = "x86_64",
    serde(try_from = "CreateBuildingDefinitionDto")
)]
pub struct BuildingDefinition {
    pub date: Box<str>,
    pub building: Box<str>,
    pub building_parts: Vec<Arc<BuildingPartEPD>>,
    pub site_energy: Vec<ConstructionSiteEnergy>,
}

#[cfg(target_arch = "x86_64")]
impl TryFrom<CreateBuildingDefinitionDto> for BuildingDefinition {
    type Error = anyhow::Error;

    fn try_from(request: CreateBuildingDefinitionDto) -> Result<Self, Self::Error> {
        let date = request.definition.date;
        let building = request.definition.building;

        let building_parts: HashMap<&str, Arc<BuildingPartEPD>> = request
            .building_part_dpps
            .iter()
            .map(|mapping| {
                mapping
                    .get_vc()
                    .map(|vc| (mapping.did.as_str(), vc.credential_subject.epd.clone()))
            })
            .try_collect()?;

        let building_parts = request
            .definition
            .building_part_dids
            .iter()
            .map(
                |BuildingPartDppDto {
                     building_part_dpp_did,
                 }| {
                    building_parts
                        .get(&**building_part_dpp_did)
                        .cloned()
                        .ok_or(anyhow!(
                            "No Dpp found for building part {building_part_dpp_did}"
                        ))
                },
            )
            .try_collect()?;

        let energy_gwp_map: HashMap<&str, u32> = request
            .site_energy_dpps
            .iter()
            .map(|mapping| {
                mapping
                    .get_vc()
                    .map(|vc| (mapping.did.as_str(), vc.credential_subject.gwp))
            })
            .try_collect()?;

        let site_energy = request
            .definition
            .site_energy_dids
            .iter()
            .map(|ConstructionSiteEnergyDto { amount, dpp_did }| {
                energy_gwp_map
                    .get(dpp_did.as_ref())
                    .map(|&gwp| ConstructionSiteEnergy {
                        gwp,
                        amount: *amount,
                    })
                    .ok_or(anyhow!("No Dpp found for SiteEnergy {dpp_did}"))
            })
            .try_collect()?;

        Ok(BuildingDefinition {
            date,
            building,
            building_parts,
            site_energy,
        })
    }
}

#[cfg(target_arch = "x86_64")]
#[derive(Deserialize)]
struct CreateBuildingDefinitionDto {
    definition: BuildingDefinitionDto,
    #[serde(rename = "buildingPartDpps")]
    building_part_dpps: Vec<DidMappingDto<ZkBuildingPartEPD>>,
    #[serde(rename = "siteEnergyDpps")]
    site_energy_dpps: Vec<DidMappingDto<EnergyDpp>>,
}

#[cfg(target_arch = "x86_64")]
#[derive(Deserialize)]
struct BuildingDefinitionDto {
    date: Box<str>,
    building: Box<str>,
    #[serde(rename = "buildingParts")]
    building_part_dids: Vec<BuildingPartDppDto>,
    #[serde(rename = "siteEnergy")]
    site_energy_dids: Vec<ConstructionSiteEnergyDto>,
}

#[cfg(target_arch = "x86_64")]
#[derive(Deserialize)]
struct BuildingPartDppDto {
    #[serde(rename = "buildingPartDppDid")]
    building_part_dpp_did: Box<str>,
}

#[cfg(target_arch = "x86_64")]
#[derive(Deserialize)]
struct ConstructionSiteEnergyDto {
    amount: u32,
    #[serde(rename = "dppDid")]
    dpp_did: Box<str>,
}

#[cfg(target_arch = "x86_64")]
#[derive(Deserialize)]
struct EnergyDpp {
    gwp: u32,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct BuildingEpd {
    pub date: Box<str>,
    pub building: Box<str>,
    pub a15_gwp: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ZkBuildingEpd {
    #[serde(flatten)]
    epd: BuildingEpd,
    zkp: Box<str>,
}

impl VerifiedEpd<BuildingEpd> for ZkBuildingEpd {
    fn get_zkp(&self) -> &str {
        &self.zkp
    }

    fn get_epd(&self) -> &BuildingEpd {
        &self.epd
    }

    fn from_result(epd: BuildingEpd, zkp: Box<str>) -> Self {
        Self { epd, zkp }
    }
}

/*#[cfg(test)]
mod test {
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn parse_create_building_part() {
        let path: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "src/test_files/testCreateBPZKP.json",
        ]
        .iter()
        .collect();
        let json_data = fs::read_to_string(path).expect("Unable to read test vector file.");
        let (input1, input2, input3): (
            CreateBuildingPartRequestDTO,
            CreateBuildingPartRequestDTO,
            CreateBuildingPartRequestDTO,
        ) = serde_json::from_str(&*json_data).expect("JSON was not well-formatted");

        #[cfg(feature = "guest-methods")]
        {
            assert!(BuildingPartDefinition::try_from(input1).is_ok());
            assert!(BuildingPartDefinition::try_from(input2).is_ok());
            assert!(BuildingPartDefinition::try_from(input3).is_err());
        }
    }

    #[test]
    fn print_building_part_epd() {
        let bp_epd = BuildingPartEPD {
            date: "2024-10-01".to_string(),
            building: "Building 1".to_string(),
            building_part_id: "xfe13293".to_string(),
            a14_gwp: 10,
        };

        let zk_bp_epd = ZkBuildingPartEPD {
            epd: bp_epd,
            zkp: "zkproof".into(),
        };

        assert_eq!(
            format!(
                "{}",
                serde_json::to_string(&zk_bp_epd).expect("Serialisation to JSON failed.")
            ),
            r#"{"date":"2024-10-01","building":"Building 1","building_part_id":"xfe13293","a14_gwp":10,"zkp":"zkproof"}"#
        );
    }
}
*/
