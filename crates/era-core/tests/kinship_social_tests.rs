use era_core::engine::SimulationKernel;
use era_core::social::KinshipRelation;

#[test]
fn test_family_tree_generation_and_era_fertility() {
    // 1920年代出生：应当生成多子女大家庭（父母 + 1至3个兄姊）
    let sim_1920 = SimulationKernel::new(1001, Some(1920));
    let tree_1920 = &sim_1920.protagonist.family_tree;

    assert!(tree_1920.len() >= 2, "Must at least have parents");
    let has_father = tree_1920.iter().any(|m| m.relation == KinshipRelation::Father);
    let has_mother = tree_1920.iter().any(|m| m.relation == KinshipRelation::Mother);
    assert!(has_father, "Must have father in family tree");
    assert!(has_mother, "Must have mother in family tree");

    // 1990年代出生：独生子女国策下出生通常无同辈兄姊
    let sim_1990 = SimulationKernel::new(2002, Some(1990));
    let tree_1990 = &sim_1990.protagonist.family_tree;
    assert_eq!(tree_1990.len(), 2, "1990s birth should typically be only-child with father and mother");
}

#[test]
fn test_friendship_system_and_natural_social_decay() {
    let mut sim = SimulationKernel::new(42, Some(1950));

    // 步进到 7 岁，应当结识儿时发小
    while sim.protagonist.age < 7 && !sim.is_game_over {
        sim.step_year();
    }

    if !sim.is_game_over {
        assert_eq!(sim.protagonist.friends.len(), 1, "Should make childhood friend at age 7");
        let initial_intimacy = sim.protagonist.friends[0].intimacy;
        assert!(initial_intimacy >= 80, "Initial friendship intimacy should be warm (>=80)");

        // 再步进 3 年，不主动维系
        for _ in 0..3 {
            sim.step_year();
        }

        // 验证自然淡化：亲密度每年减少 2 点
        let decayed_intimacy = sim.protagonist.friends[0].intimacy;
        assert!(
            decayed_intimacy < initial_intimacy,
            "Friendship without interaction should naturally decay over time"
        );

        // 主动维系互动
        let friend_id = sim.protagonist.friends[0].id;
        let res = sim.interact_friend(friend_id);
        assert!(res.is_ok(), "Interacting with living friend should succeed");

        let boosted_state = res.unwrap();
        let boosted_intimacy = boosted_state.protagonist.friends[0].intimacy;
        assert!(
            boosted_intimacy > decayed_intimacy,
            "Interacting should restore intimacy: {} vs {}",
            boosted_intimacy,
            decayed_intimacy
        );
    }
}

#[test]
fn test_parental_aging_and_bereavement_log() {
    let mut sim = SimulationKernel::new(8888, Some(1930));

    // 连续模拟至终局或年迈
    let final_state = sim.run_to_end();

    // 检查终局时年长亲属（如父母）的自然寿命演进
    let father = final_state.protagonist.family_tree.iter().find(|m| m.relation == KinshipRelation::Father);
    if let Some(f) = father {
        if !f.is_alive {
            assert!(f.death_year.is_some());
            assert!(f.death_year.unwrap() >= 1930);
            // 确认年鉴中留下了丧亲的庄重记录
            let has_bereavement_log = final_state.protagonist.log.iter().any(|entry| {
                entry.title.contains("尊亲") || entry.title.contains("丧父") || entry.content.contains("父亲")
            });
            assert!(has_bereavement_log, "Should record father passing away in life logs");
        }
    }
}
