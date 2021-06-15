use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    pub uuid:           String,
    pub player_count:   u64,
    pub mem_mb:         u64,
    pub mc_version:     f64,
    pub os:             String,
    pub java_version:   u32,
    pub timezone:       String
}