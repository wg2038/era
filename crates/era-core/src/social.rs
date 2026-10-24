use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use crate::demographics::calculate_annual_mortality_rate;
use crate::models::LifeLogEntry;
use crate::prng::DeterministicRng;

/// 亲属亲缘关系类别
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum KinshipRelation {
    Father,         // 父亲
    Mother,         // 母亲
    ElderBrother,   // 兄长
    YoungerBrother, // 弟弟
    ElderSister,    // 姊姊
    YoungerSister,  // 妹妹
    Spouse,         // 配偶
    Son,            // 儿子
    Daughter,       // 女儿
}

impl KinshipRelation {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Father => "父亲",
            Self::Mother => "母亲",
            Self::ElderBrother => "兄长",
            Self::YoungerBrother => "胞弟",
            Self::ElderSister => "长姊",
            Self::YoungerSister => "胞妹",
            Self::Spouse => "配偶",
            Self::Son => "长子",
            Self::Daughter => "长女",
        }
    }
}

/// 家族谱系实体
#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct KinshipMember {
    pub id: u32,
    pub name: String,
    pub relation: KinshipRelation,
    pub birth_year: u16,
    pub age: u16,
    pub is_alive: bool,
    pub death_year: Option<u16>,
    pub intimacy: i32, // 亲密度 0..=100
    pub occupation: String,
}

/// 朋辈社交实体（朋友系统：不维系会随岁月自然淡化）
#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Friend {
    pub id: u32,
    pub name: String,
    pub acquaintance_year: u16,
    pub age: u16,
    pub is_alive: bool,
    pub intimacy: i32, // 亲密度 0..=100（每年自然衰减 2-4 点）
    pub tag: String,   // "儿时发小", "同窗挚友", "车间工友", "生意伙伴"
}

/// 生成主角初生时的原始家族树（父母与同胞兄弟姐妹）
pub fn generate_initial_family_tree(
    birth_year: u16,
    rng: &mut DeterministicRng,
) -> Vec<KinshipMember> {
    let mut family = Vec::with_capacity(6);

    // 1. 父亲（初生时通常 22–34 岁）
    let father_age = rng.gen_range_inclusive_i32(22, 34) as u16;
    let father_birth = birth_year.saturating_sub(father_age);
    let father_occupations = ["耕读农户", "小镇木匠", "学堂教书先生", "行商账房", "码头脚夫", "手工艺人"];
    let f_occ = father_occupations[rng.gen_range_u32(0, father_occupations.len() as u32) as usize];

    family.push(KinshipMember {
        id: 101,
        name: "父亲".to_string(),
        relation: KinshipRelation::Father,
        birth_year: father_birth,
        age: father_age,
        is_alive: true,
        death_year: None,
        intimacy: 85,
        occupation: f_occ.to_string(),
    });

    // 2. 母亲（初生时通常 20–30 岁）
    let mother_age = rng.gen_range_inclusive_i32(20, 30) as u16;
    let mother_birth = birth_year.saturating_sub(mother_age);
    let mother_occupations = ["勤苦操持家务", "缫丝刺绣", "采桑持家", "织布女红", "乡村里弄操持"];
    let m_occ = mother_occupations[rng.gen_range_u32(0, mother_occupations.len() as u32) as usize];

    family.push(KinshipMember {
        id: 102,
        name: "母亲".to_string(),
        relation: KinshipRelation::Mother,
        birth_year: mother_birth,
        age: mother_age,
        is_alive: true,
        death_year: None,
        intimacy: 90,
        occupation: m_occ.to_string(),
    });

    // 3. 兄弟姐妹：按年代生育率生成兄姊（1980年独生前通常有1-3个兄姊）
    let sibling_count = if birth_year < 1980 {
        rng.gen_range_inclusive_i32(1, 3) as usize
    } else {
        0 // 80年代后独生为主
    };

    let sibling_names = ["长兄", "次兄", "长姊", "二姊"];
    for i in 0..sibling_count {
        let sib_age = rng.gen_range_inclusive_i32(2, 8) as u16;
        let sib_birth = birth_year.saturating_sub(sib_age);
        let is_brother = rng.gen_bool(0.5);
        let relation = if is_brother {
            KinshipRelation::ElderBrother
        } else {
            KinshipRelation::ElderSister
        };

        family.push(KinshipMember {
            id: 103 + i as u32,
            name: sibling_names[i % sibling_names.len()].to_string(),
            relation,
            birth_year: sib_birth,
            age: sib_age,
            is_alive: true,
            death_year: None,
            intimacy: 75,
            occupation: "尚在幼年".to_string(),
        });
    }

    family
}

/// 家族年鉴时钟推进：推演父母亲属生老病死与亲情羁绊
pub fn tick_family_tree(
    family: &mut [KinshipMember],
    current_year: u16,
    region: &str,
    rng: &mut DeterministicRng,
    logs: &mut Vec<LifeLogEntry>,
    protagonist_age: u16,
) {
    for member in family.iter_mut() {
        if !member.is_alive {
            continue;
        }

        member.age += 1;

        // 依据亲属当前年龄与年代生命表判定健康与寿命
        let (mortality_rate, _) =
            calculate_annual_mortality_rate(member.age, current_year, region, 65);

        if rng.next_f64() < mortality_rate {
            member.is_alive = false;
            member.death_year = Some(current_year);

            let log_title = match member.relation {
                KinshipRelation::Father => "尊亲见背（痛失生父）",
                KinshipRelation::Mother => "慈母长逝（痛断肝肠）",
                KinshipRelation::ElderBrother | KinshipRelation::YoungerBrother => "骨肉伤逝（痛别同胞）",
                KinshipRelation::ElderSister | KinshipRelation::YoungerSister => "骨肉伤逝（痛别姊妹）",
                KinshipRelation::Spouse => "丧偶之悲（悼亡抚孤）",
                _ => "痛失亲人",
            };

            logs.push(LifeLogEntry {
                year: current_year,
                age: protagonist_age,
                title: log_title.to_string(),
                content: format!(
                    "{}年，你的{}在{}岁时不幸离世，家门缟素，悲不自胜。",
                    current_year,
                    member.relation.title(),
                    member.age
                ),
                category: "destiny".to_string(),
            });
        }
    }
}

/// 朋友圈岁时演进：若无互动则逐年自然衰减（不维系会淡）
pub fn tick_friends(
    friends: &mut Vec<Friend>,
    current_year: u16,
    region: &str,
    rng: &mut DeterministicRng,
) {
    for friend in friends.iter_mut() {
        if !friend.is_alive {
            continue;
        }

        friend.age += 1;

        // 朋友寿命判定
        let (mortality_rate, _) =
            calculate_annual_mortality_rate(friend.age, current_year, region, 60);
        if rng.next_f64() < mortality_rate {
            friend.is_alive = false;
        } else {
            // 社交自然衰减：每年亲密度自然减少 2 点，降至 10 分以下即逐渐形同陌路
            friend.intimacy = (friend.intimacy - 2).max(0);
        }
    }
}

/// 主动维系朋友关系（互动：聚谈/书信/赠礼）
pub fn nurture_friendship(friend: &mut Friend, boost: i32) {
    if friend.is_alive {
        friend.intimacy = (friend.intimacy + boost).clamp(0, 100);
    }
}

/// 主动关爱照料亲属（互动：侍奉/问安/寄家书）
pub fn nurture_family(member: &mut KinshipMember, boost: i32) {
    if member.is_alive {
        member.intimacy = (member.intimacy + boost).clamp(0, 100);
    }
}
