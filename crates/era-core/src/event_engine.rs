use crate::data::{
    get_era_data, LocationData, RawChoiceOption, RawEventCard,
};
use crate::models::{
    ActiveChoiceOption, ActiveEventPrompt, LifeLogEntry, PendingDelayedConsequence, Person,
    PersonalityTrait,
};
use crate::prng::DeterministicRng;

/// 延迟后果轮询处理：在每年开端检查是否有到期的延迟因果
pub fn tick_delayed_consequences(person: &mut Person, current_year: u16) {
    let mut still_pending = Vec::new();
    let pending = std::mem::take(&mut person.pending_delayed);

    for item in pending {
        if item.target_year <= current_year {
            // 到期显现
            person
                .attributes
                .apply_delta(item.dh, item.dk, item.dm, item.di);
            person.wealth_ppp = (person.wealth_ppp + item.d_wealth).max(0);

            person.log.push(LifeLogEntry {
                year: current_year,
                age: person.age,
                title: "岁月回响".to_string(),
                content: item.message,
                category: "delayed".to_string(),
            });
        } else {
            still_pending.push(item);
        }
    }

    person.pending_delayed = still_pending;
}

/// 匹配当前年份和主角所适用的时代宏观事件（宏观历史层）
pub fn find_applicable_era_event<'a>(
    person: &Person,
    location: &LocationData,
) -> Option<&'a RawEventCard> {
    let data = get_era_data();
    for evt in &data.events {
        if evt.tier != "era" {
            continue;
        }
        if person.triggered_events.contains(&evt.id) {
            continue;
        }

        let tr = &evt.triggers;
        if let Some(min_y) = tr.min_year {
            if person.current_year < min_y {
                continue;
            }
        }
        if let Some(max_y) = tr.max_year {
            if person.current_year > max_y {
                continue;
            }
        }
        if let Some(min_a) = tr.min_age {
            if person.age < min_a {
                continue;
            }
        }
        if let Some(max_a) = tr.max_age {
            if person.age > max_a {
                continue;
            }
        }
        if let Some(ref regions) = tr.regions {
            if !regions.contains(&location.region) {
                continue;
            }
        }

        return Some(evt);
    }
    None
}

/// 匹配当前年龄与履历所适用的人生抉择事件（人生分支层）
pub fn find_applicable_life_event<'a>(
    person: &Person,
    location: &LocationData,
    rng: &mut DeterministicRng,
) -> Option<&'a RawEventCard> {
    let data = get_era_data();
    let mut candidates: Vec<&'a RawEventCard> = Vec::new();

    for evt in &data.events {
        if evt.tier != "life" {
            continue;
        }
        if person.triggered_events.contains(&evt.id) {
            continue;
        }

        let tr = &evt.triggers;
        if let Some(min_y) = tr.min_year {
            if person.current_year < min_y {
                continue;
            }
        }
        if let Some(max_y) = tr.max_year {
            if person.current_year > max_y {
                continue;
            }
        }
        if let Some(min_a) = tr.min_age {
            if person.age < min_a {
                continue;
            }
        }
        if let Some(max_a) = tr.max_age {
            if person.age > max_a {
                continue;
            }
        }
        if let Some(ref regions) = tr.regions {
            if !regions.contains(&location.region) {
                continue;
            }
        }

        candidates.push(evt);
    }

    if candidates.is_empty() {
        None
    } else {
        let chosen_idx = rng.gen_range_u32(0, candidates.len() as u32) as usize;
        Some(candidates[chosen_idx])
    }
}

/// 构建呈现给前端的待决断事件交互对象
pub fn build_active_event_prompt(evt: &RawEventCard) -> ActiveEventPrompt {
    let choices = if let Some(ref list) = evt.choices {
        list.iter()
            .map(|opt| ActiveChoiceOption {
                id: opt.id.clone(),
                text: opt.text.clone(),
                dh: opt.effects.health.unwrap_or(0),
                dk: opt.effects.knowledge.unwrap_or(0),
                dm: opt.effects.mood.unwrap_or(0),
                di: opt.effects.insight.unwrap_or(0),
                d_wealth: opt.effects.wealth_ppp.unwrap_or(0),
                risk: opt.risk,
            })
            .collect()
    } else {
        Vec::new()
    };

    ActiveEventPrompt {
        event_id: evt.id.clone(),
        tier: evt.tier.clone(),
        title: evt.title.clone(),
        narrative: evt.narrative.clone(),
        is_crossroads: evt.is_major_crossroads(),
        choices,
    }
}

/// 依据性格倾向为次要事件自动评分并选择最佳选项（快进模式使用）
pub fn pick_choice_by_personality<'a>(
    personality: PersonalityTrait,
    choices: &'a [RawChoiceOption],
) -> &'a RawChoiceOption {
    assert!(!choices.is_empty());

    let mut best_idx = 0;
    let mut best_score = f64::NEG_INFINITY;

    for (idx, opt) in choices.iter().enumerate() {
        let dh = opt.effects.health.unwrap_or(0) as f64;
        let dk = opt.effects.knowledge.unwrap_or(0) as f64;
        let dm = opt.effects.mood.unwrap_or(0) as f64;
        let di = opt.effects.insight.unwrap_or(0) as f64;
        let dw = opt.effects.wealth_ppp.unwrap_or(0) as f64;
        let risk = opt.risk.unwrap_or(0.0);

        let score = match personality {
            // 尚儒笃学：推崇学识与见识，稳健求进
            PersonalityTrait::Scholar => dk * 2.2 + di * 1.6 + dm * 0.8 + dw * 0.5 - risk * 35.0,
            // 敢为人先：追逐机遇与财富，敢于承担适度风险
            PersonalityTrait::Enterprising => dw * 2.2 + di * 1.5 + dk * 0.8 - risk * 8.0,
            // 务实稳健：注重健康身躯与家当保全，厌恶风险
            PersonalityTrait::Pragmatic => dh * 2.0 + dw * 1.5 + dm * 0.8 - risk * 50.0,
            // 豁达随缘：追求心情畅达与体魄安宁
            PersonalityTrait::Harmonious => dm * 2.5 + dh * 1.5 + di * 0.8 - risk * 20.0,
            // 家国热血：重视理想抱负、见识与精神满足
            PersonalityTrait::Idealist => dk * 1.8 + di * 1.8 + dm * 1.2 - risk * 15.0,
        };

        if score > best_score {
            best_score = score;
            best_idx = idx;
        }
    }

    &choices[best_idx]
}

/// 执行指定的事件选择并结算即时后果、风险摇骰与延迟队列
pub fn resolve_event_choice(
    person: &mut Person,
    evt: &RawEventCard,
    choice_id: &str,
    rng: &mut DeterministicRng,
) -> Result<(), String> {
    let choices = evt
        .choices
        .as_ref()
        .ok_or_else(|| format!("Event {} has no choices to select from", evt.id))?;

    let opt = choices
        .iter()
        .find(|c| c.id == choice_id)
        .ok_or_else(|| format!("Choice {} not found in event {}", choice_id, evt.id))?;

    // 1. 结算直接即时属性变更（资源互相挤压，不设纯收益）
    let dh = opt.effects.health.unwrap_or(0);
    let dk = opt.effects.knowledge.unwrap_or(0);
    let dm = opt.effects.mood.unwrap_or(0);
    let di = opt.effects.insight.unwrap_or(0);
    let dw = opt.effects.wealth_ppp.unwrap_or(0);

    person.attributes.apply_delta(dh, dk, dm, di);
    person.wealth_ppp = (person.wealth_ppp + dw).max(0);

    // 2. 判定风险率（如果带风险且检定失败，产生额外代价）
    let mut risk_narrative = String::new();
    if let Some(risk) = opt.risk {
        if risk > 0.0 && rng.gen_bool(risk) {
            // 风险兑现：心情受挫，小幅扣减健康
            person.attributes.apply_delta(-8, 0, -12, 4);
            risk_narrative = "【行事遭遇波折，徒生烦虑】".to_string();
        }
    }

    // 3. 注册延迟后果（3-10年显现）
    if let Some(ref delayed_list) = opt.delayed_effects {
        for d in delayed_list {
            person.pending_delayed.push(PendingDelayedConsequence {
                source_event_id: evt.id.clone(),
                target_year: person.current_year + d.delay_years,
                dh: d.effects.health.unwrap_or(0),
                dk: d.effects.knowledge.unwrap_or(0),
                dm: d.effects.mood.unwrap_or(0),
                di: d.effects.insight.unwrap_or(0),
                d_wealth: d.effects.wealth_ppp.unwrap_or(0),
                message: d.message.clone(),
            });
        }
    }

    // 4. 记录履历与生命年鉴
    person.triggered_events.push(evt.id.clone());

    let content = if risk_narrative.is_empty() {
        format!("{}: 选择了【{}】。", evt.narrative, opt.text)
    } else {
        format!("{}: 选择了【{}】。{}", evt.narrative, opt.text, risk_narrative)
    };

    person.log.push(LifeLogEntry {
        year: person.current_year,
        age: person.age,
        title: evt.title.clone(),
        content,
        category: evt.tier.clone(),
    });

    Ok(())
}

/// 执行无选项事件（命运层叙事或时代纯通告事件）
pub fn apply_narrative_event(person: &mut Person, evt: &RawEventCard) {
    if let Some(ref eff) = evt.effects {
        let dh = eff.health.unwrap_or(0);
        let dk = eff.knowledge.unwrap_or(0);
        let dm = eff.mood.unwrap_or(0);
        let di = eff.insight.unwrap_or(0);
        let dw = eff.wealth_ppp.unwrap_or(0);

        person.attributes.apply_delta(dh, dk, dm, di);
        person.wealth_ppp = (person.wealth_ppp + dw).max(0);
    }

    person.triggered_events.push(evt.id.clone());

    person.log.push(LifeLogEntry {
        year: person.current_year,
        age: person.age,
        title: evt.title.clone(),
        content: evt.narrative.clone(),
        category: evt.tier.clone(),
    });
}
