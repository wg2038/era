use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

/// 角色基础属性包（四维属性，取值范围 0..=100）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Attributes {
    pub health: i32,    // 健康
    pub knowledge: i32, // 学识
    pub mood: i32,      // 心情
    pub insight: i32,   // 见识
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            health: 70,
            knowledge: 20,
            mood: 60,
            insight: 15,
        }
    }
}

impl Attributes {
    pub fn clamp(&mut self) {
        self.health = self.health.clamp(0, 100);
        self.knowledge = self.knowledge.clamp(0, 100);
        self.mood = self.mood.clamp(0, 100);
        self.insight = self.insight.clamp(0, 100);
    }

    pub fn apply_delta(&mut self, dh: i32, dk: i32, dm: i32, di: i32) {
        self.health += dh;
        self.knowledge += dk;
        self.mood += dm;
        self.insight += di;
        self.clamp();
    }
}

/// 性别
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum Gender {
    Male,
    Female,
}

/// 历史死因枚举（严格对齐近代中国病理与社会历史）
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum CauseOfDeath {
    InfantMortality(String), // 襁褓早夭（0岁：惊厥/伤寒/破伤风/胎弱）
    ChildMortality(String),  // 幼童早夭（1-5岁：麻疹/天花/百日咳/痢疾）
    Famine(String),          // 饥荒与重度营养不良
    WartimeCasualty(String), // 战乱兵燹波及
    Epidemic(String),        // 烈性传染病（霍乱/鼠疫/疟疾/血吸虫）
    Accident(String),        // 生产与生活意外（落水/坍塌/车马祸）
    OrganDisease(String),    // 器质性重疾（中风/心疾/肺痈）
    OldAge,                  // 寿终正寝（高龄自然机能衰退）
    SurviveTo2025,           // 平安生存至2025年（终局）
}

impl CauseOfDeath {
    pub fn description(&self) -> String {
        match self {
            Self::InfantMortality(s) => format!("襁褓夭折：{s}"),
            Self::ChildMortality(s) => format!("幼年夭折：{s}"),
            Self::Famine(s) => format!("遭遇大饥荒：{s}"),
            Self::WartimeCasualty(s) => format!("战火罹难：{s}"),
            Self::Epidemic(s) => format!("染疾不治：{s}"),
            Self::Accident(s) => format!("飞来横祸：{s}"),
            Self::OrganDisease(s) => format!("病逝：{s}"),
            Self::OldAge => "无疾而终，享尽天年".to_string(),
            Self::SurviveTo2025 => "跨越百年风云，见证时代至今".to_string(),
        }
    }
}

/// 人生年鉴记录
#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct LifeLogEntry {
    pub year: u16,
    pub age: u16,
    pub title: String,
    pub content: String,
    pub category: String, // "destiny", "era", "life", "health"
}

/// 个人生命实体
#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Person {
    pub id: u32,
    pub name: String,
    pub gender: Gender,
    pub birth_year: u16,
    pub birth_location_id: String,
    pub current_year: u16,
    pub age: u16,
    pub is_alive: bool,
    pub death_year: Option<u16>,
    pub death_age: Option<u16>,
    pub cause_of_death: Option<CauseOfDeath>,
    pub attributes: Attributes,
    pub log: Vec<LifeLogEntry>,
}

/// 完整模拟状态快照
#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct SimulationState {
    pub seed: u64,
    pub current_year: u16,
    pub active_index: usize, // 0 为主角，1..=4 为同乡同龄四人
    pub protagonist: Person,
    pub peers: Vec<Person>,
    pub is_game_over: bool,
    pub can_switch_peer: bool, // 是否允许早夭移魂
}
