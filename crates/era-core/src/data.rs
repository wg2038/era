use std::collections::HashMap;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use crate::prng::DeterministicRng;

pub const RAW_ERA_DATA_JSON: &str = include_str!("../../../packages/data/dist/era-data.json");

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalNameEntry {
    pub start_year: u16,
    pub end_year: u16,
    pub name: String,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct LocationData {
    pub id: String,
    pub current_name: String,
    pub region: String,
    pub population_weight: HashMap<String, f64>,
    pub historical_names: Vec<HistoricalNameEntry>,
    pub climate: String,
    pub source: String,
    pub confidence: String,
}

impl LocationData {
    /// 计算指定年份的人口权重（基于历史普查锚点线性插值）
    pub fn weight_at_year(&self, year: u16) -> f64 {
        let anchors = [(1900, 1.0), (1930, 1.0), (1953, 1.0), (1982, 1.0), (1999, 1.0)];
        let mut resolved_anchors: Vec<(u16, f64)> = anchors
            .iter()
            .map(|(y, def)| {
                let w = self
                    .population_weight
                    .get(&y.to_string())
                    .copied()
                    .unwrap_or(*def);
                (*y, w)
            })
            .collect();
        resolved_anchors.sort_by_key(|a| a.0);

        if year <= resolved_anchors.first().unwrap().0 {
            return resolved_anchors.first().unwrap().1;
        }
        if year >= resolved_anchors.last().unwrap().0 {
            return resolved_anchors.last().unwrap().1;
        }

        for window in resolved_anchors.windows(2) {
            let (y0, w0) = window[0];
            let (y1, w1) = window[1];
            if year >= y0 && year <= y1 {
                let ratio = (year - y0) as f64 / (y1 - y0) as f64;
                return w0 + (w1 - w0) * ratio;
            }
        }
        1.0
    }

    /// 获取特定年份的历史地名与备注
    pub fn get_historical_name(&self, year: u16) -> (&str, Option<&str>) {
        for entry in &self.historical_names {
            if year >= entry.start_year && year <= entry.end_year {
                return (&entry.name, entry.note.as_deref());
            }
        }
        // 若超出区间，取最接近的首末项
        if let Some(first) = self.historical_names.first() {
            if year < first.start_year {
                return (&first.name, first.note.as_deref());
            }
        }
        if let Some(last) = self.historical_names.last() {
            return (&last.name, last.note.as_deref());
        }
        (&self.current_name, None)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawEffects {
    pub health: Option<i32>,
    pub knowledge: Option<i32>,
    pub mood: Option<i32>,
    pub insight: Option<i32>,
    #[serde(alias = "wealthPPP", alias = "wealthPpp")]
    pub wealth_ppp: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawDelayedEffect {
    pub delay_years: u16,
    pub effects: RawEffects,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawChoiceOption {
    pub id: String,
    pub text: String,
    pub effects: RawEffects,
    pub delayed_effects: Option<Vec<RawDelayedEffect>>,
    pub risk: Option<f64>,
    pub source: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawTriggerConditions {
    pub min_year: Option<u16>,
    pub max_year: Option<u16>,
    pub min_age: Option<u16>,
    pub max_age: Option<u16>,
    pub regions: Option<Vec<String>>,
    pub requires_trait: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawEventCard {
    pub id: String,
    pub tier: String, // "destiny", "era", "life"
    pub title: String,
    pub narrative: String,
    pub triggers: RawTriggerConditions,
    pub choices: Option<Vec<RawChoiceOption>>,
    pub effects: Option<RawEffects>,
    pub delayed_effects: Option<Vec<RawDelayedEffect>>,
    pub source: String,
    pub confidence: String,
}

impl RawEventCard {
    /// 判断是否属于人生关口（在快进模式下仍必须暂停等待玩家抉择）
    pub fn is_major_crossroads(&self) -> bool {
        match self.id.as_str() {
            "evt_era_1919_may_fourth"
            | "evt_era_1937_resistance_war"
            | "evt_era_1949_prc_founding"
            | "evt_era_1977_gaokao_resumed"
            | "evt_era_1978_reform_opening"
            | "evt_era_1992_southern_tour"
            | "evt_life_education_crossroad"
            | "evt_life_career_branch"
            | "evt_life_marriage_decision"
            | "evt_life_real_estate_boom" => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawLifeTableEntry {
    pub decade: u16,
    pub infant_mortality_rate: f64,
    pub child_mortality_rate: f64,
    pub base_crude_death_rate: f64,
    pub life_expectancy: f64,
    pub source: String,
    pub confidence: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawEraData {
    pub version: String,
    pub build_timestamp: String,
    pub locations: Vec<LocationData>,
    pub events: Vec<RawEventCard>,
    pub life_tables: Vec<RawLifeTableEntry>,
}

static EMBEDDED_DATA: OnceLock<RawEraData> = OnceLock::new();

/// 获取全局静态解析的年代数据库
pub fn get_era_data() -> &'static RawEraData {
    EMBEDDED_DATA.get_or_init(|| {
        serde_json::from_str(RAW_ERA_DATA_JSON).expect("Failed to deserialize embedded era-data.json")
    })
}

/// 依据出生年份与人口加权采样出生地点（涵盖中国各大地理大区代表城市与乡镇）
pub fn sample_birth_location(birth_year: u16, rng: &mut DeterministicRng) -> LocationData {
    let data = get_era_data();
    let weights: Vec<f64> = data
        .locations
        .iter()
        .map(|loc| loc.weight_at_year(birth_year))
        .collect();

    let idx = rng.choose_weighted(&weights);
    data.locations[idx].clone()
}

/// 查找指定地点
pub fn find_location(id: &str) -> Option<&'static LocationData> {
    get_era_data().locations.iter().find(|loc| loc.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_data_loading() {
        let data = get_era_data();
        assert_eq!(data.locations.len(), 30);
        assert_eq!(data.events.len(), 40);
        assert_eq!(data.life_tables.len(), 10);
    }

    #[test]
    fn test_beijing_historical_names() {
        let beijing = find_location("loc_beijing").expect("Beijing should exist");
        let (name1905, _) = beijing.get_historical_name(1905);
        assert_eq!(name1905, "京师顺天府");

        let (name1920, _) = beijing.get_historical_name(1920);
        assert_eq!(name1920, "北京");

        let (name1935, _) = beijing.get_historical_name(1935);
        assert_eq!(name1935, "北平");

        let (name1955, _) = beijing.get_historical_name(1955);
        assert_eq!(name1955, "北京");
    }

    #[test]
    fn test_shenzhen_historical_names() {
        let sz = find_location("loc_shenzhen").expect("Shenzhen should exist");
        let (name1905, _) = sz.get_historical_name(1905);
        assert_eq!(name1905, "新安县");

        let (name1930, _) = sz.get_historical_name(1930);
        assert_eq!(name1930, "宝安县");

        let (name1990, _) = sz.get_historical_name(1990);
        assert_eq!(name1990, "深圳市");
    }

    #[test]
    fn test_population_weight_sampling() {
        let mut rng = DeterministicRng::new(12345);
        let loc = sample_birth_location(1930, &mut rng);
        assert!(!loc.id.is_empty());
    }
}
