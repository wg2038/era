use era_core::engine::SimulationKernel;
use era_core::models::{LifeFocus, SocialActionType};

#[test]
fn test_reboot_yearly_focus_and_turn_based_mechanics() {
    let mut sim = SimulationKernel::new(12345, Some(1980));
    let state = sim.get_state();
    assert_eq!(state.current_year, 1980);
    assert_eq!(state.protagonist.age, 0);

    // 1. Advance year with Study focus
    let s1 = sim.advance_year(LifeFocus::Study).unwrap();
    assert_eq!(s1.current_year, 1981);
    assert_eq!(s1.protagonist.age, 1);
    assert_eq!(s1.current_focus, Some(LifeFocus::Study));

    // Resolve any pending choice if one triggered
    if let Some(ref prompt) = s1.pending_choice {
        sim.make_choice(&prompt.choices[0].id).unwrap();
    }

    // 2. Advance year with Livelihood focus
    let s2 = sim.advance_year(LifeFocus::Livelihood).unwrap();
    assert_eq!(s2.current_year, 1982);
    assert_eq!(s2.protagonist.age, 2);
    assert_eq!(s2.current_focus, Some(LifeFocus::Livelihood));
}

#[test]
fn test_reboot_milestone_reflections_pauses() {
    // Start at 1980 where infant mortality is much lower
    let mut sim = SimulationKernel::new(42, Some(1980));
    while sim.get_state().protagonist.age < 5 && !sim.get_state().is_game_over {
        if let Some(ref prompt) = sim.get_state().pending_choice {
            sim.make_choice(&prompt.choices[0].id).unwrap();
        }
        if let Some(ref m) = sim.get_state().active_milestone {
            sim.submit_milestone_quote(&m.default_quote).unwrap();
        }
        let _ = sim.advance_year(LifeFocus::Health);
    }

    assert_eq!(sim.get_state().protagonist.age, 5);
    assert!(sim.get_state().active_milestone.is_some());
    let m = sim.get_state().active_milestone.unwrap();
    assert_eq!(m.age, 5);
    assert_eq!(m.question, "“五岁的你，心里最想要一件什么东西？”");

    // Attempting to advance year while milestone is pending should fail
    let err = sim.advance_year(LifeFocus::Study);
    assert!(err.is_err());

    // Submit quote
    let sealed = sim.submit_milestone_quote("想要娘蒸的热馒头").unwrap();
    assert!(sealed.active_milestone.is_none());
    assert_eq!(sealed.protagonist.milestone_quotes.len(), 1);
    assert_eq!(sealed.protagonist.milestone_quotes[0].quote, "想要娘蒸的热馒头");
}

#[test]
fn test_reboot_social_actions_limit_and_variety() {
    let mut sim = SimulationKernel::new(99999, Some(1940));
    let s = sim.get_state();
    assert_eq!(s.protagonist.actions_remaining_this_year, 2);

    // 1. Visit mother (id 102)
    let s1 = sim.interact_social_relative(102, SocialActionType::Visit).unwrap();
    assert_eq!(s1.protagonist.actions_remaining_this_year, 1);

    // 2. Remit father (id 101)
    let s2 = sim.interact_social_relative(101, SocialActionType::Remit).unwrap();
    assert_eq!(s2.protagonist.actions_remaining_this_year, 0);

    // 3. Exceeded action limit in the same year
    let s3 = sim.interact_social_relative(102, SocialActionType::Letter);
    assert!(s3.is_err());

    // Advance year, limit resets to 2
    let s4 = sim.advance_year(LifeFocus::Family).unwrap();
    assert_eq!(s4.protagonist.actions_remaining_this_year, 2);
}
