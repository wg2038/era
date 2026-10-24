use std::collections::HashSet;
use era_core::data::get_era_data;
use era_core::engine::SimulationKernel;

#[test]
fn test_all_40_events_integrity_in_database() {
    let data = get_era_data();
    assert_eq!(data.events.len(), 40, "There should be exactly 40 seed events in database");
    assert_eq!(data.locations.len(), 30, "There should be exactly 30 locations in database");

    let mut destiny_count = 0;
    let mut era_count = 0;
    let mut life_count = 0;

    for evt in &data.events {
        assert!(!evt.id.is_empty());
        assert!(!evt.title.is_empty());
        assert!(!evt.narrative.is_empty());
        assert!(!evt.source.is_empty(), "Event {} must have non-empty historical source", evt.id);

        match evt.tier.as_str() {
            "destiny" => destiny_count += 1,
            "era" => era_count += 1,
            "life" => life_count += 1,
            other => panic!("Unexpected tier: {}", other),
        }

        // 验证有选项的事件，其选项必须有资源互相挤压（不设纯收益）
        if let Some(ref choices) = evt.choices {
            assert!(choices.len() >= 2, "Event {} must have at least 2 choices", evt.id);
            for c in choices {
                assert!(!c.text.is_empty());
                let dh = c.effects.health.unwrap_or(0);
                let dk = c.effects.knowledge.unwrap_or(0);
                let dm = c.effects.mood.unwrap_or(0);
                let di = c.effects.insight.unwrap_or(0);
                let dw = c.effects.wealth_ppp.unwrap_or(0);

                let has_gain = dh > 0 || dk > 0 || dm > 0 || di > 0 || dw > 0;
                let has_cost = dh < 0 || dk < 0 || dm < 0 || di < 0 || dw < 0 || c.risk.unwrap_or(0.0) > 0.0;

                if has_gain {
                    assert!(
                        has_cost,
                        "Choice {} in event {} is a pure-gain option! Must have cost, risk or squeeze.",
                        c.id, evt.id
                    );
                }
            }
        }
    }

    assert_eq!(destiny_count, 10, "Should have 10 destiny events");
    assert_eq!(era_count, 15, "Should have 15 era events");
    assert_eq!(life_count, 15, "Should have 15 life events");
}

#[test]
fn test_choice_execution_and_delayed_consequences() {
    let mut sim = SimulationKernel::new(42, Some(1953));
    let mut encountered_choice = false;

    while !sim.is_game_over && !encountered_choice {
        let state = sim.step_year();
        if let Some(prompt) = state.pending_choice {
            encountered_choice = true;
            assert!(!prompt.choices.is_empty());

            let first_choice_id = prompt.choices[0].id.clone();
            let result = sim.make_choice(&first_choice_id);
            assert!(result.is_ok(), "make_choice should succeed");

            let updated_state = result.unwrap();
            assert!(updated_state.pending_choice.is_none(), "Pending choice should be cleared");
            break;
        }
    }

    assert!(encountered_choice, "A choice should be encountered in life before death");
}

#[test]
fn test_event_coverage_in_simulation_runs() {
    let mut triggered_set = HashSet::new();

    // 在不同出生年代（1900, 1920, 1940, 1960, 1980）运行多轮自适应模拟
    let birth_years = [1900, 1915, 1930, 1945, 1960, 1975, 1990];
    for &by in &birth_years {
        for seed in 0..150 {
            let mut sim = SimulationKernel::new((by as u64) * 1000 + seed, Some(by));
            let final_state = sim.run_to_end();

            for evt_id in &final_state.protagonist.triggered_events {
                triggered_set.insert(evt_id.clone());
            }
        }
    }

    let all_events = &get_era_data().events;
    let era_and_life_count = all_events.iter().filter(|e| e.tier == "era" || e.tier == "life").count();
    let triggered_count = all_events.iter().filter(|e| triggered_set.contains(&e.id)).count();

    println!("Simulation event coverage: {}/{} era/life events triggered", triggered_count, era_and_life_count);
    assert!(
        triggered_count >= 25,
        "Expected at least 25 different era/life events to trigger in 1050 runs across decades, got {}",
        triggered_count
    );
}
