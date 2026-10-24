use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use crate::destiny::evaluate_destiny;
use crate::models::{Attributes, CauseOfDeath, Gender, LifeLogEntry, Person, SimulationState};
use crate::prng::DeterministicRng;

/// 预设地点基础信息（供内核加权采样）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocationSample {
    pub id: String,
    pub name: String,
    pub region: String,
    pub base_weight: f64,
}

/// 默认基础地点列表（包含中国各大地理大区代表性城镇与乡村）
pub fn get_default_locations() -> Vec<LocationSample> {
    vec![
        LocationSample {
            id: "loc_beijing".to_string(),
            name: "北京".to_string(),
            region: "华北".to_string(),
            base_weight: 1.5,
        },
        LocationSample {
            id: "loc_shanghai".to_string(),
            name: "上海".to_string(),
            region: "华东".to_string(),
            base_weight: 2.5,
        },
        LocationSample {
            id: "loc_shenzhen".to_string(),
            name: "深圳".to_string(),
            region: "华南".to_string(),
            base_weight: 0.5,
        },
        LocationSample {
            id: "loc_wuhan".to_string(),
            name: "武汉".to_string(),
            region: "华中".to_string(),
            base_weight: 2.0,
        },
        LocationSample {
            id: "loc_chengdu".to_string(),
            name: "成都".to_string(),
            region: "西南".to_string(),
            base_weight: 2.2,
        },
        LocationSample {
            id: "loc_xian".to_string(),
            name: "西安".to_string(),
            region: "西北".to_string(),
            base_weight: 1.4,
        },
        LocationSample {
            id: "loc_shenyang".to_string(),
            name: "沈阳".to_string(),
            region: "东北".to_string(),
            base_weight: 1.8,
        },
    ]
}

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
    pub birth_location: LocationSample,
    pub protagonist: Person,
    pub peers: Vec<Person>, // 4名同乡同龄人
    pub active_index: usize,
    pub is_game_over: bool,
    pub can_switch_peer: bool,
}

impl SimulationKernel {
    /// 基于种子初始化全新人生局
    pub fn new(seed: u64, fixed_birth_year: Option<u16>) -> Self {
        let mut rng = DeterministicRng::new(seed);

        let birth_year = fixed_birth_year.unwrap_or_else(|| {
            // 随机出生在 1900–1999 年
            rng.gen_range_inclusive_i32(1900, 1999) as u16
        });

        // 依据人口权重采样出生地
        let locations = get_default_locations();
        let weights: Vec<f64> = locations.iter().map(|l| l.base_weight).collect();
        let loc_idx = rng.choose_weighted(&weights);
        let birth_location = locations[loc_idx].clone();

        // 创建主角
        let protagonist = Self::create_person(
            0,
            birth_year,
            &birth_location.id,
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
                &birth_location.id,
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
        }
    }

    fn create_person(
        id: u32,
        birth_year: u16,
        location_id: &str,
        gender: Gender,
        name: &str,
        rng: &mut DeterministicRng,
    ) -> Person {
        let init_health = rng.gen_range_inclusive_i32(65, 80);
        let init_knowledge = rng.gen_range_inclusive_i32(15, 25);
        let init_mood = rng.gen_range_inclusive_i32(55, 75);
        let init_insight = rng.gen_range_inclusive_i32(10, 20);

        let attributes = Attributes {
            health: init_health,
            knowledge: init_knowledge,
            mood: init_mood,
            insight: init_insight,
        };

        Person {
            id,
            name: name.to_string(),
            gender,
            birth_year,
            birth_location_id: location_id.to_string(),
            current_year: birth_year,
            age: 0,
            is_alive: true,
            death_year: None,
            death_age: None,
            cause_of_death: None,
            attributes,
            log: vec![LifeLogEntry {
                year: birth_year,
                age: 0,
                title: "降生人世".to_string(),
                content: format!("{birth_year}年，你诞生在中华大地上。"),
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

    /// 获取当前操纵的角色可变引用
    pub fn active_person_mut(&mut self) -> &mut Person {
        if self.active_index == 0 {
            &mut self.protagonist
        } else {
            &mut self.peers[self.active_index - 1]
        }
    }

    /// 切换为“同年同县出生的另外四人”之一继续游玩（早夭移魂）
    pub fn switch_to_peer(&mut self, peer_index: usize) -> bool {
        // 只有在早夭且目标同乡健在时方可切换
        if !self.can_switch_peer || peer_index >= self.peers.len() {
            return false;
        }

        if !self.peers[peer_index].is_alive {
            return false;
        }

        self.active_index = peer_index + 1;
        self.can_switch_peer = false;
        self.is_game_over = false;
        true
    }

    /// 推进一年（核心时钟滴答）
    pub fn step_year(&mut self) -> SimulationState {
        if self.is_game_over {
            return self.get_state();
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

        // 判断是否平安抵达 2025 年
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

        // 执行当前活跃角色的命运层判定
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
        }

        self.get_state()
    }

    /// 获取当前完整的快照状态
    pub fn get_state(&self) -> SimulationState {
        SimulationState {
            seed: self.rng.seed(),
            current_year: self.active_person().current_year,
            active_index: self.active_index,
            protagonist: self.protagonist.clone(),
            peers: self.peers.clone(),
            is_game_over: self.is_game_over,
            can_switch_peer: self.can_switch_peer,
        }
    }

    /// 持续模拟直至终结（死亡或抵达2025年）
    pub fn run_to_end(&mut self) -> SimulationState {
        while !self.is_game_over {
            self.step_year();
        }
        self.get_state()
    }

    /// 蒙特卡洛年代模拟与对齐检验
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
                if decade + da >= 2025 {
                    reached_2025 += 1;
                }
            } else {
                reached_2025 += 1;
            }

            if final_age >= 60.0 {
                reached_60 += 1;
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
