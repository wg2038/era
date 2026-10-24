use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use crate::currency::format_currency;
use crate::data::{
    get_era_data, sample_birth_location, LocationData,
};
use crate::destiny::evaluate_destiny;
use crate::event_engine::{
    apply_narrative_event, build_active_event_prompt, find_applicable_era_event,
    find_applicable_life_event, pick_choice_by_personality, resolve_event_choice,
    tick_delayed_consequences,
};
use crate::models::{
    ActiveEventPrompt, Attributes, CauseOfDeath, Gender, LifeLogEntry, Person, PersonalityTrait,
    SimulationState,
};
use crate::prng::DeterministicRng;

/// 蒙特卡洛单个年代统计报告
#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct MonteCarloDecadeReport {
    pub decade: u16,
    pub sample_count: usize,
    pub avg_lifespan: f64,
    pub benchmark_expected_lifespan: f64,
    pub lifespan_deviation_pct: f64,
    pub infant_mortality_rate: f64,
    pub benchmark_imr: f64,
    pub imr_deviation_pct: f64,
    pub under_5_mortality_rate: f64,
    pub reached_60_pct: f64,
    pub reached_2025_pct: f64,
}

/// 核心模拟器状态机
pub struct SimulationKernel {
    pub rng: DeterministicRng,
    pub birth_year: u16,
    pub birth_location: LocationData,
    pub protagonist: Person,
    pub peers: Vec<Person>, // 4名同乡同龄人
    pub active_index: usize,
    pub is_game_over: bool,
    pub can_switch_peer: bool,
    pub pending_choice: Option<ActiveEventPrompt>,
}

impl SimulationKernel {
    /// 基于种子初始化全新人生局
    pub fn new(seed: u64, fixed_birth_year: Option<u16>) -> Self {
        let mut rng = DeterministicRng::new(seed);

        let birth_year = fixed_birth_year.unwrap_or_else(|| {
            // 随机出生在 1900–1999 年
            rng.gen_range_inclusive_i32(1900, 1999) as u16
        });

        // 依据当年各省县人口加权采样出生地（覆盖全中国30个代表性地点）
        let birth_location = sample_birth_location(birth_year, &mut rng);

        // 创建主角
        let protagonist = Self::create_person(
            0,
            birth_year,
            &birth_location,
            Gender::Male,
            "主角",
            &mut rng,
        );

        // 创建同年同县出生的另外 4 人
        let mut peers = Vec::with_capacity(4);
        let peer_names = ["同乡甲（李氏）", "同乡乙（张氏）", "同乡丙（赵氏）", "同乡丁（王氏）"];
        for (i, name) in peer_names.iter().enumerate() {
            let gender = if rng.gen_bool(0.5) {
                Gender::Male
            } else {
                Gender::Female
            };
            let peer = Self::create_person(
                (i + 1) as u32,
                birth_year,
                &birth_location,
                gender,
                name,
                &mut rng,
            );
            peers.push(peer);
        }

        Self {
            rng,
            birth_year,
            birth_location,
            protagonist,
            peers,
            active_index: 0,
            is_game_over: false,
            can_switch_peer: false,
            pending_choice: None,
        }
    }

    fn create_person(
        id: u32,
        birth_year: u16,
        location: &LocationData,
        gender: Gender,
        name: &str,
        rng: &mut DeterministicRng,
    ) -> Person {
        let init_health = rng.gen_range_inclusive_i32(65, 80);
        let init_knowledge = rng.gen_range_inclusive_i32(15, 25);
        let init_mood = rng.gen_range_inclusive_i32(55, 75);
        let init_insight = rng.gen_range_inclusive_i32(10, 20);
        let init_wealth = rng.gen_range_inclusive_i32(30, 60);

        let personality = match rng.gen_range_inclusive_i32(0, 4) {
            0 => PersonalityTrait::Scholar,
            1 => PersonalityTrait::Enterprising,
            2 => PersonalityTrait::Pragmatic,
            3 => PersonalityTrait::Harmonious,
            _ => PersonalityTrait::Idealist,
        };

        let attributes = Attributes {
            health: init_health,
            knowledge: init_knowledge,
            mood: init_mood,
            insight: init_insight,
        };

        let (hist_name, note) = location.get_historical_name(birth_year);
        let birth_desc = if hist_name != location.current_name {
            if let Some(n) = note {
                format!("{birth_year}年，你降生在{hist_name}（今{}）。（{}）", location.current_name, n)
            } else {
                format!("{birth_year}年，你降生在{hist_name}（今{}）。", location.current_name)
            }
        } else {
            format!("{birth_year}年，你降生在{}。", location.current_name)
        };

        Person {
            id,
            name: name.to_string(),
            gender,
            birth_year,
            birth_location_id: location.id.clone(),
            current_year: birth_year,
            age: 0,
            is_alive: true,
            death_year: None,
            death_age: None,
            cause_of_death: None,
            attributes,
            wealth_ppp: init_wealth,
            personality,
            pending_delayed: Vec::new(),
            triggered_events: Vec::new(),
            log: vec![LifeLogEntry {
                year: birth_year,
                age: 0,
                title: "降生人世".to_string(),
                content: birth_desc,
                category: "destiny".to_string(),
            }],
        }
    }

    /// 获取当前操纵的角色引用
    pub fn active_person(&self) -> &Person {
        if self.active_index == 0 {
            &self.protagonist
        } else {
            &self.peers[self.active_index - 1]
        }
    }

    /// 获取当前操纵的角色与随机数发生器的互斥双借用（满足借用检查器互斥字段借用规则）
    pub fn active_person_and_rng_mut(&mut self) -> (&mut Person, &mut DeterministicRng) {
        let rng = &mut self.rng;
        let person = if self.active_index == 0 {
            &mut self.protagonist
        } else {
            &mut self.peers[self.active_index - 1]
        };
        (person, rng)
    }

    /// 切换为“同年同县出生的另外四人”之一继续游玩（早夭移魂）
    pub fn switch_to_peer(&mut self, peer_index: usize) -> bool {
        if !self.can_switch_peer || peer_index >= self.peers.len() {
            return false;
        }

        if !self.peers[peer_index].is_alive {
            return false;
        }

        self.active_index = peer_index + 1;
        self.can_switch_peer = false;
        self.is_game_over = false;
        self.pending_choice = None;
        true
    }

    /// 做出抉择响应当前待决断事件
    pub fn make_choice(&mut self, choice_id: &str) -> Result<SimulationState, String> {
        let prompt = self
            .pending_choice
            .as_ref()
            .ok_or_else(|| "No pending event choice to make".to_string())?;

        let event_id = prompt.event_id.clone();
        let data = get_era_data();
        let raw_evt = data
            .events
            .iter()
            .find(|e| e.id == event_id)
            .ok_or_else(|| format!("Event {} not found in database", event_id))?;

        let (active, rng) = self.active_person_and_rng_mut();
        resolve_event_choice(active, raw_evt, choice_id, rng)?;

        // 抉择完毕，清除悬挂状态
        self.pending_choice = None;
        Ok(self.get_state())
    }

    /// 推进一年（核心时钟滴答）
    pub fn step_year(&mut self) -> SimulationState {
        if self.is_game_over {
            return self.get_state();
        }

        // 若当前有等待选择的事件且调用者直接步进，自动依据性格倾向决断，防止无限空转
        if let Some(ref prompt) = self.pending_choice {
            let event_id = prompt.event_id.clone();
            let data = get_era_data();
            if let Some(raw_evt) = data.events.iter().find(|e| e.id == event_id) {
                if let Some(ref choices) = raw_evt.choices {
                    let personality = self.active_person().personality;
                    let picked = pick_choice_by_personality(personality, choices);
                    let choice_id = picked.id.clone();
                    let (active, rng) = self.active_person_and_rng_mut();
                    let _ = resolve_event_choice(active, raw_evt, &choice_id, rng);
                }
            }
            self.pending_choice = None;
        }

        let region = self.birth_location.region.clone();

        // 1. 推进同乡存活者（背景模拟）
        for peer in self.peers.iter_mut() {
            if peer.is_alive {
                if peer.current_year >= 2025 {
                    peer.is_alive = true;
                    peer.cause_of_death = Some(CauseOfDeath::SurviveTo2025);
                } else {
                    let res = evaluate_destiny(peer, &region, &mut self.rng);
                    if res.died {
                        peer.is_alive = false;
                        peer.death_year = Some(peer.current_year);
                        peer.death_age = Some(peer.age);
                        peer.cause_of_death = res.cause;
                    } else {
                        peer.age += 1;
                        peer.current_year += 1;
                    }
                }
            }
        }

        // 2. 推进当前活跃主角
        let active = if self.active_index == 0 {
            &mut self.protagonist
        } else {
            &mut self.peers[self.active_index - 1]
        };

        // 2.1 检查并生效延迟后果（3-10年显现的因果回响）
        tick_delayed_consequences(active, active.current_year);

        // 2.2 判断是否平安抵达 2025 年
        if active.current_year >= 2025 {
            active.is_alive = true;
            active.cause_of_death = Some(CauseOfDeath::SurviveTo2025);
            active.log.push(LifeLogEntry {
                year: 2025,
                age: active.age,
                title: "见证盛世".to_string(),
                content: "你跨越百年风雨，于2025年回望百年沧桑巨变。".to_string(),
                category: "destiny".to_string(),
            });
            self.is_game_over = true;
            self.can_switch_peer = false;
            return self.get_state();
        }

        // 2.3 执行命运层生死大限判定
        let res = evaluate_destiny(active, &region, &mut self.rng);

        let mut died = false;
        let mut age_at_death = 0;
        if res.died {
            died = true;
            active.is_alive = false;
            active.death_year = Some(active.current_year);
            active.death_age = Some(active.age);
            active.cause_of_death = res.cause.clone();
            age_at_death = active.age;

            if let Some(msg) = res.narrative {
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: active.age,
                    title: "天命难违".to_string(),
                    content: msg,
                    category: "destiny".to_string(),
                });
            }
        } else {
            active.age += 1;
            active.current_year += 1;
        }

        if died {
            self.is_game_over = true;
            // 若在未成年（<18岁）夭折，且同乡有健在者，激活“早夭移魂”通道
            let has_living_peer = self.peers.iter().any(|p| p.is_alive);
            if age_at_death < 18 && has_living_peer {
                self.can_switch_peer = true;
            }
            return self.get_state();
        }

        // 2.4 若存活，进入时代层 (Era) 判定
        if let Some(era_evt) = find_applicable_era_event(active, &self.birth_location) {
            if era_evt.choices.is_some() {
                self.pending_choice = Some(build_active_event_prompt(era_evt));
                return self.get_state();
            } else {
                apply_narrative_event(active, era_evt);
            }
        } else {
            // 2.5 若本年未触发时代大事件，进入人生层 (Life) 判定
            if let Some(life_evt) =
                find_applicable_life_event(active, &self.birth_location, &mut self.rng)
            {
                if life_evt.choices.is_some() {
                    self.pending_choice = Some(build_active_event_prompt(life_evt));
                    return self.get_state();
                } else {
                    apply_narrative_event(active, life_evt);
                }
            }
        }

        self.get_state()
    }

    /// 快进模式自适应步进：次要事件按性格倾向自动决断，仅在关键岔路口（is_crossroads）暂停
    pub fn step_year_auto(&mut self) -> SimulationState {
        if self.is_game_over {
            return self.get_state();
        }

        // 1. 若当前已经有悬挂事件待选择
        if let Some(ref prompt) = self.pending_choice {
            if prompt.is_crossroads {
                // 属于重大人生关口，坚决暂停等待玩家亲决
                return self.get_state();
            } else {
                let event_id = prompt.event_id.clone();
                let data = get_era_data();
                if let Some(raw_evt) = data.events.iter().find(|e| e.id == event_id) {
                    if let Some(ref choices) = raw_evt.choices {
                        let personality = self.active_person().personality;
                        let picked = pick_choice_by_personality(personality, choices);
                        let choice_id = picked.id.clone();
                        let (active, rng) = self.active_person_and_rng_mut();
                        let _ = resolve_event_choice(active, raw_evt, &choice_id, rng);
                    }
                }
                self.pending_choice = None;
            }
        }

        // 2. 执行步进
        self.step_year();

        // 3. 步进后若新触发了事件且非重大关口，自动按性格抉择以实现流畅快进
        if let Some(ref prompt) = self.pending_choice {
            if !prompt.is_crossroads {
                let event_id = prompt.event_id.clone();
                let data = get_era_data();
                if let Some(raw_evt) = data.events.iter().find(|e| e.id == event_id) {
                    if let Some(ref choices) = raw_evt.choices {
                        let personality = self.active_person().personality;
                        let picked = pick_choice_by_personality(personality, choices);
                        let choice_id = picked.id.clone();
                        let (active, rng) = self.active_person_and_rng_mut();
                        let _ = resolve_event_choice(active, raw_evt, &choice_id, rng);
                    }
                }
                self.pending_choice = None;
            }
        }

        self.get_state()
    }

    /// 连续模拟直至人生终局（自动按性格倾向做全部抉择，供蒙特卡洛及极速模式）
    pub fn run_to_end(&mut self) -> SimulationState {
        while !self.is_game_over {
            if let Some(ref prompt) = self.pending_choice {
                let event_id = prompt.event_id.clone();
                let data = get_era_data();
                if let Some(raw_evt) = data.events.iter().find(|e| e.id == event_id) {
                    if let Some(ref choices) = raw_evt.choices {
                        let personality = self.active_person().personality;
                        let picked = pick_choice_by_personality(personality, choices);
                        let choice_id = picked.id.clone();
                        let (active, rng) = self.active_person_and_rng_mut();
                        let _ = resolve_event_choice(active, raw_evt, &choice_id, rng);
                    }
                }
                self.pending_choice = None;
            }
            self.step_year_auto();
        }
        self.get_state()
    }

    /// 获取当前完整的快照状态
    pub fn get_state(&self) -> SimulationState {
        let active = self.active_person();
        let (hist_name, _) = self
            .birth_location
            .get_historical_name(self.birth_year);
        let currency = format_currency(active.current_year, active.wealth_ppp);

        SimulationState {
            seed: self.rng.seed(),
            current_year: active.current_year,
            active_index: self.active_index,
            protagonist: self.protagonist.clone(),
            peers: self.peers.clone(),
            is_game_over: self.is_game_over,
            can_switch_peer: self.can_switch_peer,
            pending_choice: self.pending_choice.clone(),
            birth_historical_name: hist_name.to_string(),
            birth_current_name: self.birth_location.current_name.clone(),
            current_currency: currency,
        }
    }

    /// 蒙特卡洛统计检验工具（保持时期生命表校准）
    pub fn run_monte_carlo(decade: u16, count: usize) -> MonteCarloDecadeReport {
        use crate::demographics::get_life_table_for_year;

        let benchmark_table = get_life_table_for_year(decade);
        let mut total_lifespan: f64 = 0.0;
        let mut infant_deaths = 0;
        let mut under_5_deaths = 0;
        let mut reached_60 = 0;
        let mut reached_2025 = 0;

        for i in 0..count {
            let seed = ((decade as u64) << 32) ^ (i as u64).wrapping_mul(0x9E3779B97F4A7C15);
            let mut sim = SimulationKernel::new(seed, Some(decade));

            let mut age: u16 = 0;
            let mut health = sim.protagonist.attributes.health;
            let region = sim.birth_location.region.clone();
            let mut died_age = None;

            while age <= 105 {
                let res = crate::destiny::evaluate_destiny_raw(
                    age,
                    decade, // 采用该年代截面时期生命表（Period Life Table），与历史普查标准口径严谨对齐
                    &region,
                    health,
                    &mut sim.rng,
                );
                if res.died {
                    died_age = Some(age);
                    break;
                }
                if age > 45 {
                    let aging_wear = (age - 45) / 10 + 1;
                    health = (health - aging_wear as i32).max(10);
                }
                age += 1;
            }

            let final_age = died_age.unwrap_or(age) as f64;
            total_lifespan += final_age;

            if let Some(da) = died_age {
                if da == 0 {
                    infant_deaths += 1;
                }
                if da < 5 {
                    under_5_deaths += 1;
                }
            }
            if final_age >= 60.0 {
                reached_60 += 1;
            }
            if (decade as f64 + final_age) >= 2025.0 {
                reached_2025 += 1;
            }
        }

        let avg_lifespan = total_lifespan / count as f64;
        let infant_mortality_rate = infant_deaths as f64 / count as f64;
        let under_5_mortality_rate = under_5_deaths as f64 / count as f64;
        let reached_60_pct = (reached_60 as f64 / count as f64) * 100.0;
        let reached_2025_pct = (reached_2025 as f64 / count as f64) * 100.0;

        let lifespan_deviation_pct = ((avg_lifespan - benchmark_table.expected_life_at_birth)
            / benchmark_table.expected_life_at_birth)
            * 100.0;
        let imr_deviation_pct = ((infant_mortality_rate - benchmark_table.infant_mortality)
            / benchmark_table.infant_mortality)
            * 100.0;

        MonteCarloDecadeReport {
            decade,
            sample_count: count,
            avg_lifespan,
            benchmark_expected_lifespan: benchmark_table.expected_life_at_birth,
            lifespan_deviation_pct,
            infant_mortality_rate,
            benchmark_imr: benchmark_table.infant_mortality,
            imr_deviation_pct,
            under_5_mortality_rate,
            reached_60_pct,
            reached_2025_pct,
        }
    }
}
