use era_core::engine::SimulationKernel;

#[test]
fn test_exact_determinism_across_entire_lifespan() {
    let test_seeds = [123456789u64, 987654321u64, 42u64, 20261024u64, 0xDEADBEEFu64];

    for &seed in &test_seeds {
        let mut sim_a = SimulationKernel::new(seed, Some(1930));
        let mut sim_b = SimulationKernel::new(seed, Some(1930));

        assert_eq!(sim_a.birth_location.id, sim_b.birth_location.id);
        assert_eq!(sim_a.protagonist.attributes, sim_b.protagonist.attributes);

        // 逐年对比每一步的状态
        while !sim_a.is_game_over {
            let state_a = sim_a.step_year();
            let state_b = sim_b.step_year();

            assert_eq!(state_a.current_year, state_b.current_year);
            assert_eq!(state_a.protagonist.age, state_b.protagonist.age);
            assert_eq!(state_a.protagonist.is_alive, state_b.protagonist.is_alive);
            assert_eq!(state_a.protagonist.attributes, state_b.protagonist.attributes);
            assert_eq!(state_a.protagonist.death_age, state_b.protagonist.death_age);
            assert_eq!(state_a.is_game_over, state_b.is_game_over);
        }

        assert!(sim_b.is_game_over);
    }
}

#[test]
fn test_peer_switching_on_premature_death() {
    // 寻找一个在 18 岁前夭折的种子进行测试
    let mut target_seed = None;
    for seed in 1..1000 {
        let mut sim = SimulationKernel::new(seed, Some(1910));
        let state = sim.run_to_end();
        if !state.protagonist.is_alive && state.protagonist.death_age.unwrap() < 18 {
            if state.peers.iter().any(|p| p.is_alive) {
                target_seed = Some(seed);
                break;
            }
        }
    }

    assert!(target_seed.is_some(), "Should find a seed with premature death");
    let seed = target_seed.unwrap();

    let mut sim = SimulationKernel::new(seed, Some(1910));
    while !sim.can_switch_peer {
        sim.step_year();
    }

    assert!(sim.can_switch_peer);
    // 找到第一个健在的同乡
    let peer_idx = sim.peers.iter().position(|p| p.is_alive).unwrap();
    let switch_ok = sim.switch_to_peer(peer_idx);
    assert!(switch_ok);
    assert_eq!(sim.active_index, peer_idx + 1);
    assert!(!sim.is_game_over);

    // 继续推进切换后的同乡人生直至终局
    let final_state = sim.run_to_end();
    assert!(final_state.is_game_over);
}
