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
    ActiveEventPrompt, Attributes, CauseOfDeath, Gender, LifeFocus, LifeLogEntry, MilestonePrompt,
    MilestoneQuote, Person, PersonalityTrait, SimulationState, SocialActionType,
};
use crate::prng::DeterministicRng;
use crate::social::{
    generate_initial_family_tree, tick_family_tree, tick_friends, Friend,
};

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

/// 获取特定人生节点（5、15、30、50、70 岁）的自省提问
pub fn get_milestone_prompt(age: u16, year: u16) -> Option<MilestonePrompt> {
    match age {
        5 => Some(MilestonePrompt {
            age,
            year,
            title: "童蒙初启 · 五岁纪事".to_string(),
            narrative: "你五岁了。开始记事的那年，弄堂里炊烟四起，外面的世界动荡不安，你在门槛上看蚂蚁搬家。".to_string(),
            question: "“五岁的你，心里最想要一件什么东西？”".to_string(),
            default_quote: "只想要一只带响的竹蜻蜓，还有娘蒸的热馒头。".to_string(),
        }),
        15 => Some(MilestonePrompt {
            age,
            year,
            title: "及笄弱冠 · 十五立志".to_string(),
            narrative: "十五岁，少年骨骼拔节，时代风潮激荡。你站在人生的门槛上，学业或生计催促你迅速长大。".to_string(),
            question: "“十五岁这一年，你最想成为一个怎样的人？”".to_string(),
            default_quote: "只想学得一技之长，让一家老小不再挨饿受冻。".to_string(),
        }),
        30 => Some(MilestonePrompt {
            age,
            year,
            title: "三十而立 · 风雨成家".to_string(),
            narrative: "三十岁，青年意气沉淀为柴米油盐与责任。时代的巨轮轰鸣向前，个人在其中颠簸漂泊。".to_string(),
            question: "“三十岁这一年，你最庆幸自己没有放弃什么？”".to_string(),
            default_quote: "纵使世道几番翻覆，我仍保住了一双手的手艺，守住了屋檐下的这点人间烟火。".to_string(),
        }),
        50 => Some(MilestonePrompt {
            age,
            year,
            title: "五十知命 · 沧桑回首".to_string(),
            narrative: "半百之年，鬓染霜白。走过半个世纪的波澜起伏，儿女渐渐成人，旧友日渐零落。".to_string(),
            question: "“五十岁回望半生，哪一个决定让你至今无悔？”".to_string(),
            default_quote: "不后悔在那年最艰难的时候，没有丢下亲人独自逃生。".to_string(),
        }),
        70 => Some(MilestonePrompt {
            age,
            year,
            title: "古稀之年 · 浮生若梦".to_string(),
            narrative: "七十岁，步履蹒跚，坐在夕阳斜照的院角。新世纪的楼宇林立，百年的往事如云烟掠过。".to_string(),
            question: "“七十岁坐对残阳，你想对这个世道说些什么？”".to_string(),
            default_quote: "看尽人间悲欢离合，这天下终于太平丰衣足食了，足矣。".to_string(),
        }),
        _ => None,
    }
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
    pub active_milestone: Option<MilestonePrompt>,
    pub current_focus: Option<LifeFocus>,
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
            active_milestone: None,
            current_focus: None,
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

        let family_tree = generate_initial_family_tree(birth_year, rng);

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
            family_tree,
            friends: Vec::new(),
            pending_delayed: Vec::new(),
            triggered_events: Vec::new(),
            milestone_quotes: Vec::new(),
            actions_remaining_this_year: 2,
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

    /// 亲友多类型微互动操作（寄钱、写信、探望、断联、赡养）
    pub fn interact_social_relative(
        &mut self,
        relative_id: u32,
        action: SocialActionType,
    ) -> Result<SimulationState, String> {
        let (active, _) = self.active_person_and_rng_mut();
        if active.actions_remaining_this_year == 0 {
            return Err("本年度亲友互动次数已用尽（每年最多2次）。".to_string());
        }

        let member = active
            .family_tree
            .iter_mut()
            .find(|m| m.id == relative_id)
            .ok_or_else(|| format!("未找到编号为 {} 的亲属", relative_id))?;

        if !member.is_alive {
            return Err("该亲属已故，无法互动。".to_string());
        }

        active.actions_remaining_this_year -= 1;
        let rel_title = member.relation.title();

        match action {
            SocialActionType::Remit => {
                let cost = if active.wealth_ppp >= 5 { 5 } else { active.wealth_ppp };
                active.wealth_ppp -= cost;
                member.intimacy = (member.intimacy + 15).min(100);
                active.attributes.apply_delta(0, 0, 4, 1);
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: active.age,
                    title: format!("寄资济家 · {}", rel_title),
                    content: format!("你省吃俭用，给【{}】汇去家资银两（{} PPP），对方回信满纸感激，甚感慰藉。", rel_title, cost),
                    category: "life".to_string(),
                });
            }
            SocialActionType::Letter => {
                member.intimacy = (member.intimacy + 8).min(100);
                active.attributes.apply_delta(0, 0, 5, 2);
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: active.age,
                    title: format!("寄书问安 · {}", rel_title),
                    content: format!("你铺纸研墨，给【{}】寄去长信，细叙冷暖生活与平生感怀，见字如晤。", rel_title),
                    category: "life".to_string(),
                });
            }
            SocialActionType::Visit => {
                let cost = if active.wealth_ppp >= 2 { 2 } else { active.wealth_ppp };
                active.wealth_ppp -= cost;
                member.intimacy = (member.intimacy + 18).min(100);
                active.attributes.apply_delta(1, 0, 8, 2);
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: active.age,
                    title: format!("登门探望 · {}", rel_title),
                    content: format!("你专程抽空探望【{}】，促膝长谈，执手相顾，虽耗盘缠，心头温情笃厚。", rel_title),
                    category: "life".to_string(),
                });
            }
            SocialActionType::Sever => {
                member.intimacy = 0;
                active.attributes.apply_delta(0, 0, -3, 1);
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: active.age,
                    title: format!("渐行渐远 · {}", rel_title),
                    content: format!("因观念殊途或岁月隔阂，你决意与【{}】断绝走动，从此音信稀疏。", rel_title),
                    category: "life".to_string(),
                });
            }
            SocialActionType::Support => {
                let cost = if active.wealth_ppp >= 10 { 10 } else { active.wealth_ppp };
                active.wealth_ppp -= cost;
                member.intimacy = (member.intimacy + 25).min(100);
                active.attributes.apply_delta(0, 0, 10, 3);
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: active.age,
                    title: format!("尽孝赡养 · {}", rel_title),
                    content: format!("你尽心侍奉供养【{}】，延医抓药、晨昏问安，竭尽人伦大孝之义。", rel_title),
                    category: "life".to_string(),
                });
            }
        }

        Ok(self.get_state())
    }

    /// 朋友多类型微互动操作（寄钱、写信、探望、断联、相助）
    pub fn interact_social_friend(
        &mut self,
        friend_id: u32,
        action: SocialActionType,
    ) -> Result<SimulationState, String> {
        let (active, _) = self.active_person_and_rng_mut();
        if active.actions_remaining_this_year == 0 {
            return Err("本年度亲友互动次数已用尽（每年最多2次）。".to_string());
        }

        let friend = active
            .friends
            .iter_mut()
            .find(|f| f.id == friend_id)
            .ok_or_else(|| format!("未找到编号为 {} 的好友", friend_id))?;

        if !friend.is_alive {
            return Err("故人已逝，徒留追忆。".to_string());
        }

        active.actions_remaining_this_year -= 1;
        let f_name = friend.name.clone();

        match action {
            SocialActionType::Remit => {
                let cost = if active.wealth_ppp >= 4 { 4 } else { active.wealth_ppp };
                active.wealth_ppp -= cost;
                friend.intimacy = (friend.intimacy + 15).min(100);
                active.attributes.apply_delta(0, 0, 5, 2);
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: active.age,
                    title: format!("解囊济友 · {}", f_name),
                    content: format!("得知好友【{}】手头拮据，你慷慨解囊资助，情义深重。", f_name),
                    category: "life".to_string(),
                });
            }
            SocialActionType::Letter => {
                friend.intimacy = (friend.intimacy + 10).min(100);
                active.attributes.apply_delta(0, 0, 6, 2);
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: active.age,
                    title: format!("书信寄情 · {}", f_name),
                    content: format!("你给好友【{}】写了一封长信，遥寄相思与平生近况，友情愈笃。", f_name),
                    category: "life".to_string(),
                });
            }
            SocialActionType::Visit => {
                let cost = if active.wealth_ppp >= 2 { 2 } else { active.wealth_ppp };
                active.wealth_ppp -= cost;
                friend.intimacy = (friend.intimacy + 20).min(100);
                active.attributes.apply_delta(0, 0, 8, 3);
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: active.age,
                    title: format!("茶楼小聚 · {}", f_name),
                    content: format!("你与好友【{}】相约茶肆酒馆，围炉对饮，畅叙天下事，畅快淋漓。", f_name),
                    category: "life".to_string(),
                });
            }
            SocialActionType::Sever => {
                friend.intimacy = 0;
                active.attributes.apply_delta(0, 0, -2, 1);
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: active.age,
                    title: format!("割席断交 · {}", f_name),
                    content: format!("因志趣相左或世故纷扰，你与【{}】割席分道，再不相闻。", f_name),
                    category: "life".to_string(),
                });
            }
            SocialActionType::Support => {
                let cost = if active.wealth_ppp >= 6 { 6 } else { active.wealth_ppp };
                active.wealth_ppp -= cost;
                friend.intimacy = (friend.intimacy + 20).min(100);
                active.attributes.apply_delta(0, 0, 7, 2);
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: active.age,
                    title: format!("鼎力相扶 · {}", f_name),
                    content: format!("好友【{}】遇上难关，你挺身而出倾力照料相扶，患难见真情。", f_name),
                    category: "life".to_string(),
                });
            }
        }

        Ok(self.get_state())
    }

    /// 主动互动侍奉亲属（兼容老接口，默认探望）
    pub fn interact_relative(&mut self, relative_id: u32) -> Result<SimulationState, String> {
        let (active, _) = self.active_person_and_rng_mut();
        if active.actions_remaining_this_year == 0 {
            active.actions_remaining_this_year = 1;
        }
        self.interact_social_relative(relative_id, SocialActionType::Visit)
    }

    /// 主动拜会好友朋辈（兼容老接口，默认聚访）
    pub fn interact_friend(&mut self, friend_id: u32) -> Result<SimulationState, String> {
        let (active, _) = self.active_person_and_rng_mut();
        if active.actions_remaining_this_year == 0 {
            active.actions_remaining_this_year = 1;
        }
        self.interact_social_friend(friend_id, SocialActionType::Visit)
    }

    /// 封存人生节点心声提问
    pub fn submit_milestone_quote(&mut self, quote: &str) -> Result<SimulationState, String> {
        let prompt = self
            .active_milestone
            .as_ref()
            .ok_or_else(|| "当前没有待封存的人生节点提问。".to_string())?
            .clone();

        let final_quote = if quote.trim().is_empty() {
            prompt.default_quote.clone()
        } else {
            quote.trim().to_string()
        };

        let active = if self.active_index == 0 {
            &mut self.protagonist
        } else {
            &mut self.peers[self.active_index - 1]
        };

        active.milestone_quotes.push(MilestoneQuote {
            age: prompt.age,
            year: prompt.year,
            question: prompt.question.clone(),
            quote: final_quote.clone(),
        });

        active.log.push(LifeLogEntry {
            year: prompt.year,
            age: prompt.age,
            title: format!("人生节点心声 · {}岁", prompt.age),
            content: format!("你在此刻写下：“{}”", final_quote),
            category: "life".to_string(),
        });

        self.active_milestone = None;
        Ok(self.get_state())
    }

    /// 回合制主动推演一年（指定生命重心：读书/谋生/家人/身体/人情）
    pub fn advance_year(&mut self, focus: LifeFocus) -> Result<SimulationState, String> {
        if self.is_game_over {
            return Ok(self.get_state());
        }

        // 若当前有等待填写的反思节点，必须先封存心声
        if self.active_milestone.is_some() {
            return Err("当前处于人生关键节点，请先留下一句话封存心声。".to_string());
        }

        // 若当前有待玩家决断的重大事件，必须先做出抉择
        if self.pending_choice.is_some() {
            return Err("当前有待玩家决断的重大事件，请先做出抉择。".to_string());
        }

        self.current_focus = Some(focus);

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

        // 恢复当年亲友行动额度（每年最多2次）
        active.actions_remaining_this_year = 2;

        // 2.1 应用当年生命重心效果
        match focus {
            LifeFocus::Study => {
                active.attributes.apply_delta(-1, 5, -1, 2);
                let cost = if active.wealth_ppp > 5 { 5 } else { active.wealth_ppp };
                active.wealth_ppp -= cost;
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: active.age,
                    title: "【年度重心 · 读书求学】".to_string(),
                    content: "你整年埋首于书本课业之中，诵读求知，学识与见识渐长，但伏案苦读亦耗费心力家资。".to_string(),
                    category: "life".to_string(),
                });
            }
            LifeFocus::Livelihood => {
                active.attributes.apply_delta(-2, 0, -2, 1);
                active.wealth_ppp += 10;
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: active.age,
                    title: "【年度重心 · 帮贴谋生】".to_string(),
                    content: "这一年你奔波营生，吃尽了人间烟火的苦头，好在积攒下微薄积蓄（购买力+10 PPP）。".to_string(),
                    category: "life".to_string(),
                });
            }
            LifeFocus::Family => {
                active.attributes.apply_delta(1, 0, 6, 0);
                let cost = if active.wealth_ppp > 2 { 2 } else { active.wealth_ppp };
                active.wealth_ppp -= cost;
                for m in active.family_tree.iter_mut() {
                    if m.is_alive {
                        m.intimacy = (m.intimacy + 6).min(100);
                    }
                }
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: active.age,
                    title: "【年度重心 · 顾全家人】".to_string(),
                    content: "你将大部分精力倾注在照料双亲与家中老小上，晨昏侍奉，虽未有外财，家庭和睦温情脉脉。".to_string(),
                    category: "life".to_string(),
                });
            }
            LifeFocus::Health => {
                active.attributes.apply_delta(6, 0, 3, 0);
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: active.age,
                    title: "【年度重心 · 调养身体】".to_string(),
                    content: "你注重节制起居，调理体魄，避风寒、进粗羹，体魄渐趋强健，避过了这一年的几场时疫。".to_string(),
                    category: "health".to_string(),
                });
            }
            LifeFocus::Social => {
                active.attributes.apply_delta(0, 0, 4, 3);
                let cost = if active.wealth_ppp > 3 { 3 } else { active.wealth_ppp };
                active.wealth_ppp -= cost;
                for f in active.friends.iter_mut() {
                    if f.is_alive {
                        f.intimacy = (f.intimacy + 8).min(100);
                    }
                }
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: active.age,
                    title: "【年度重心 · 维系人情】".to_string(),
                    content: "你频繁与乡邻四邻、昔日同窗往来应酬，送往迎来，人情通达，在街坊间口皆有碑。".to_string(),
                    category: "life".to_string(),
                });
            }
        }

        // 2.2 检查并生效延迟后果（3-10年显现的因果回响）
        tick_delayed_consequences(active, active.current_year);

        // 2.3 判断是否平安抵达 2025 年
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
            return Ok(self.get_state());
        }

        // 2.4 执行命运层生死大限判定
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
            return Ok(self.get_state());
        }

        // 2.5 推进家族亲族与朋友圈社交时钟（亲人生老病死与朋友自然淡化）
        {
            let (active, rng) = self.active_person_and_rng_mut();
            tick_family_tree(
                &mut active.family_tree,
                active.current_year,
                &region,
                rng,
                &mut active.log,
                active.age,
            );
            tick_friends(&mut active.friends, active.current_year, &region, rng);

            // 结识新朋友（7岁儿时发小、16岁同窗挚友）
            if active.age == 7 && active.friends.is_empty() {
                let names = ["赵小栓", "陈二宝", "孙巧云", "周阿福", "林春燕"];
                let friend_name = names[rng.gen_range_u32(0, names.len() as u32) as usize];
                active.friends.push(Friend {
                    id: 201,
                    name: friend_name.to_string(),
                    acquaintance_year: active.current_year,
                    age: 7,
                    is_alive: true,
                    intimacy: 85,
                    tag: "儿时发小".to_string(),
                });
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: 7,
                    title: "结识发小".to_string(),
                    content: format!("在弄堂打谷场与古榕树下，你结识了邻家玩伴【{}】，结伴摸鱼捕雀，两小无猜。", friend_name),
                    category: "life".to_string(),
                });
            } else if active.age == 16 && active.friends.len() == 1 {
                let names = ["张志强", "王建华", "刘雪梅", "徐文斌", "杨晓红"];
                let friend_name = names[rng.gen_range_u32(0, names.len() as u32) as usize];
                active.friends.push(Friend {
                    id: 202,
                    name: friend_name.to_string(),
                    acquaintance_year: active.current_year,
                    age: 16,
                    is_alive: true,
                    intimacy: 80,
                    tag: "同窗知己".to_string(),
                });
                active.log.push(LifeLogEntry {
                    year: active.current_year,
                    age: 16,
                    title: "求知同窗".to_string(),
                    content: format!("青年立身求学之际，你与同窗【{}】志趣相投，常在油灯与晨曦下切磋学问、指点江山。", friend_name),
                    category: "life".to_string(),
                });
            }
        }

        // 2.6 检查人生节点提问（5、15、30、50、70 岁）
        if self.active_index == 0 {
            let active = &self.protagonist;
            if let Some(prompt) = get_milestone_prompt(active.age, active.current_year) {
                self.active_milestone = Some(prompt);
                return Ok(self.get_state());
            }
        }

        // 2.7 时代层与人生层事件判定
        let (active, rng) = if self.active_index == 0 {
            (&mut self.protagonist, &mut self.rng)
        } else {
            (&mut self.peers[self.active_index - 1], &mut self.rng)
        };
        let loc = &self.birth_location;

        if let Some(era_evt) = find_applicable_era_event(active, loc) {
            if era_evt.choices.is_some() {
                self.pending_choice = Some(build_active_event_prompt(era_evt));
                return Ok(self.get_state());
            } else {
                apply_narrative_event(active, era_evt);
            }
        } else if let Some(life_evt) = find_applicable_life_event(active, loc, rng) {
            if life_evt.choices.is_some() {
                self.pending_choice = Some(build_active_event_prompt(life_evt));
                return Ok(self.get_state());
            } else {
                apply_narrative_event(active, life_evt);
            }
        }

        Ok(self.get_state())
    }

    /// 推进一年（兼容接口：自动按性格决断或选择重心）
    pub fn step_year(&mut self) -> SimulationState {
        if self.is_game_over {
            return self.get_state();
        }

        // 若当前有等待选择的事件且调用者直接步进，自动依据性格倾向决断
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

        // 若处于人生节点自省中，自动以默认心声封存
        if let Some(ref prompt) = self.active_milestone {
            let def = prompt.default_quote.clone();
            let _ = self.submit_milestone_quote(&def);
        }

        let focus = match self.active_person().personality {
            PersonalityTrait::Scholar => LifeFocus::Study,
            PersonalityTrait::Enterprising => LifeFocus::Livelihood,
            PersonalityTrait::Harmonious => LifeFocus::Family,
            PersonalityTrait::Pragmatic => LifeFocus::Health,
            PersonalityTrait::Idealist => LifeFocus::Social,
        };

        let _ = self.advance_year(focus);
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

        // 若处于人生节点自省中，自动以默认心声封存以保证快进畅通
        if let Some(ref prompt) = self.active_milestone {
            let def = prompt.default_quote.clone();
            let _ = self.submit_milestone_quote(&def);
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
            active_milestone: self.active_milestone.clone(),
            current_focus: self.current_focus,
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
