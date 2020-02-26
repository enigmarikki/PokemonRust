use crate::battle::backend::BattleEvent;

use super::{prelude::*, TestMethods, TestRng};

#[test]
fn tackle_deals_damage() {
    let mut backend = battle! {
        "Rattata" 50 (max ivs, Adamant) vs "Pidgey" 50 (max ivs, Adamant)
    };

    let events = backend.process_turn("Tackle", "Tackle");

    assert_pattern!(events[0], BattleEvent::Damage {
        target: 1,
        amount: 39,
        is_critical_hit: false,
        ..
    });
    assert_pattern!(events[1], BattleEvent::Damage {
        target: 0,
        amount: 36,
        is_critical_hit: false,
        ..
    });
}

#[test]
fn tackle_does_accuracy_checks() {
    let mut backend = battle! {
        "Rattata" 3 (max ivs, Serious) vs "Pidgey" 3 (max ivs, Serious)
    };

    backend.rng.force_miss(3);
    let turn1 = backend.process_turn("Tackle", "Tackle");
    let turn2 = backend.process_turn("Tackle", "Tackle");

    assert_eq!(turn1[0], BattleEvent::Miss(0));
    assert_eq!(turn1[1], BattleEvent::Miss(1));
    assert_eq!(turn2[0], BattleEvent::Miss(0));
    assert_pattern!(turn2[1], BattleEvent::Damage { target: 0, .. });
    assert_eq!(backend.rng.get_last_miss_check_chance(), Some(100));
}
