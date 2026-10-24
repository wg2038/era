use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use crate::currency::CurrencyDisplay;
use crate::social::{Friend, KinshipMember};

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

/// 年度生命精力重心（回合制每年必选其一）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum LifeFocus {
    Study,      // 读书求学
    Livelihood, // 帮贴谋生
    Family,     // 顾全家人
    Health,     // 调养身体
    Social,     // 维系人情
}

impl LifeFocus {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Study => "读书求学",
            Self::Livelihood => "帮贴谋生",
            Self::Family => "顾全家人",
            Self::Health => "调养身体",
            Self::Social => "维系人情",
        }
    }

    pub fn from_str_id(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "study" | "读书" | "读书求学" => Some(Self::Study),
            "livelihood" | "谋生" | "帮贴谋生" => Some(Self::Livelihood),
            "family" | "家人" | "顾全家人" => Some(Self::Family),
            "health" | "身体" | "调养身体" => Some(Self::Health),
            "social" | "人情" | "维系人情" => Some(Self::Social),
            _ => None,
        }
    }
}

/// 亲友微互动类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum SocialActionType {
    Remit,   // 寄钱（寄资济困）
    Letter,  // 写信（书信寄情）
    Visit,   // 探望（登门走动）
    Sever,   // 断联（割席绝交）
    Support, // 赡养（尽孝侍奉）
}

impl SocialActionType {
    pub fn from_str_id(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "remit" | "寄钱" => Some(Self::Remit),
            "letter" | "写信" => Some(Self::Letter),
            "visit" | "探望" => Some(Self::Visit),
            "sever" | "断联" => Some(Self::Sever),
            "support" | "赡养" => Some(Self::Support),
            _ => None,
        }
    }
}

/// 性格倾向（用于快进模式下的次要抉择自动判定）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum PersonalityTrait {
    Scholar,      // 尚儒笃学（崇尚读书与见识）
    Enterprising, // 敢为人先（勇于开拓与追求财富，敢冒适度风险）
    Pragmatic,    // 务实稳健（注重健康与家财保全，厌恶风险）
    Harmonious,   // 豁达随缘（重视心情与身心平衡）
    Idealist,     // 家国热血（重视理想抱负与见识）
}

impl PersonalityTrait {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Scholar => "尚儒笃学",
            Self::Enterprising => "敢为人先",
            Self::Pragmatic => "务实稳健",
            Self::Harmonious => "豁达随缘",
            Self::Idealist => "家国热血",
        }
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

/// 待生效的延迟后果队列项（3-10年显现）
#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct PendingDelayedConsequence {
    pub source_event_id: String,
    pub target_year: u16,
    pub dh: i32,
    pub dk: i32,
    pub dm: i32,
    pub di: i32,
    pub d_wealth: i32,
    pub message: String,
}

/// 抉择分支视图
#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ActiveChoiceOption {
    pub id: String,
    pub text: String,
    pub dh: i32,
    pub dk: i32,
    pub dm: i32,
    pub di: i32,
    pub d_wealth: i32,
    pub risk: Option<f64>,
}

/// 当前待玩家做出决断的事件视图
#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ActiveEventPrompt {
    pub event_id: String,
    pub tier: String,
    pub title: String,
    pub narrative: String,
    pub is_crossroads: bool,
    pub choices: Vec<ActiveChoiceOption>,
}

/// 人生年鉴记录
#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct LifeLogEntry {
    pub year: u16,
    pub age: u16,
    pub title: String,
    pub content: String,
    pub category: String, // "destiny", "era", "life", "health", "delayed"
}

/// 人生节点反思存留记录（5、15、30、50、70 岁）
#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct MilestoneQuote {
    pub age: u16,
    pub year: u16,
    pub question: String,
    pub quote: String,
}

/// 当前待玩家作答的人生节点自省提示
#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct MilestonePrompt {
    pub age: u16,
    pub year: u16,
    pub title: String,
    pub narrative: String,
    pub question: String,
    pub default_quote: String,
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
    pub wealth_ppp: i32,                         // 内部购买力点数（PPP）
    pub personality: PersonalityTrait,           // 性格倾向
    pub family_tree: Vec<KinshipMember>,         // 家族谱系（存亡与亲密度）
    pub friends: Vec<Friend>,                    // 朋友圈（不维系会自然淡化）
    pub pending_delayed: Vec<PendingDelayedConsequence>, // 待生效延迟后果
    pub triggered_events: Vec<String>,           // 已经历的历史/人生事件ID
    pub milestone_quotes: Vec<MilestoneQuote>,   // 人生关键节点所留句子
    pub actions_remaining_this_year: u8,         // 本年度亲友微互动剩余可用次数（最多2次）
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
    pub pending_choice: Option<ActiveEventPrompt>,
    pub active_milestone: Option<MilestonePrompt>, // 人生节点反思提问暂停
    pub current_focus: Option<LifeFocus>,          // 本年度选定的人生重心
    pub birth_historical_name: String,
    pub birth_current_name: String,
    pub current_currency: CurrencyDisplay,
}
