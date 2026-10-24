use crate::demographics::calculate_annual_mortality_rate;
use crate::models::{CauseOfDeath, Person};
use crate::prng::DeterministicRng;

/// 命运层单年推演结果
pub struct DestinyResolution {
    pub died: bool,
    pub cause: Option<CauseOfDeath>,
    pub narrative: Option<String>,
}

/// 底层直接命运判定计算
pub fn evaluate_destiny_raw(
    age: u16,
    current_year: u16,
    region: &str,
    health: i32,
    rng: &mut DeterministicRng,
) -> DestinyResolution {
    let (mortality_rate, era_event) =
        calculate_annual_mortality_rate(age, current_year, region, health);

    let roll = rng.next_f64();
    if roll < mortality_rate {
        let cause = pick_cause_of_death(age, era_event, rng);
        let narrative = format!(
            "{}年，{}岁时，不幸因【{}】离世。",
            current_year,
            age,
            cause.description()
        );

        DestinyResolution {
            died: true,
            cause: Some(cause),
            narrative: Some(narrative),
        }
    } else {
        DestinyResolution {
            died: false,
            cause: None,
            narrative: None,
        }
    }
}

/// 执行角色命运层判定
pub fn evaluate_destiny(
    person: &mut Person,
    region: &str,
    rng: &mut DeterministicRng,
) -> DestinyResolution {
    let res = evaluate_destiny_raw(
        person.age,
        person.current_year,
        region,
        person.attributes.health,
        rng,
    );

    if !res.died && person.age > 45 {
        let aging_wear = (person.age - 45) / 10 + 1;
        person.attributes.health = (person.attributes.health - aging_wear as i32).max(10);
    }

    res
}

/// 根据年龄与时代背景采样死因
fn pick_cause_of_death(
    age: u16,
    era_event: Option<&'static str>,
    rng: &mut DeterministicRng,
) -> CauseOfDeath {
    if let Some(event) = era_event {
        if rng.gen_bool(0.70) {
            if event.contains("饥荒") || event.contains("年馑") || event.contains("匮乏") {
                return CauseOfDeath::Famine(event.to_string());
            }
            if event.contains("战争") || event.contains("战火") || event.contains("战时") {
                return CauseOfDeath::WartimeCasualty(event.to_string());
            }
            if event.contains("鼠疫") || event.contains("流行") {
                return CauseOfDeath::Epidemic(event.to_string());
            }
        }
    }

    match age {
        0 => {
            let infant_causes = [
                "突发高热惊厥不退",
                "新生儿破伤风",
                "急性重症痢疾",
                "先天胎弱夭折",
                "冬季重症肺炎",
            ];
            let idx = rng.gen_range_u32(0, infant_causes.len() as u32) as usize;
            CauseOfDeath::InfantMortality(infant_causes[idx].to_string())
        }
        1..=4 => {
            let child_causes = [
                "出麻疹并发肺炎",
                "感染天花痘疮",
                "急性百日咳窒息",
                "细菌性重度痢疾",
                "意外落井落水",
            ];
            let idx = rng.gen_range_u32(0, child_causes.len() as u32) as usize;
            CauseOfDeath::ChildMortality(child_causes[idx].to_string())
        }
        5..=17 => {
            let teen_causes = [
                "罹患急症伤寒",
                "溺水或山崖意外",
                "恶性疟疾（打摆子）",
                "重度营养不良衰竭",
                "突发盲肠炎（阑尾炎）穿孔",
            ];
            let idx = rng.gen_range_u32(0, teen_causes.len() as u32) as usize;
            CauseOfDeath::Accident(teen_causes[idx].to_string())
        }
        18..=69 => {
            let roll = rng.next_f64();
            if roll < 0.45 {
                CauseOfDeath::OrganDisease("突发心肺急症或中风".to_string())
            } else if roll < 0.65 {
                CauseOfDeath::Accident("工农业生产或交通意外".to_string())
            } else if roll < 0.90 {
                CauseOfDeath::Epidemic("感染烈性传染病不治".to_string())
            } else {
                CauseOfDeath::Famine("常年操劳积劳成疾".to_string())
            }
        }
        _ => {
            if age >= 80 && rng.gen_bool(0.65) {
                CauseOfDeath::OldAge
            } else {
                CauseOfDeath::OrganDisease("器官衰竭，寿尽瞑目".to_string())
            }
        }
    }
}
