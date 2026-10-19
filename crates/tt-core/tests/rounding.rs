//! R17/R18: rounding rules and the rounding function (plan.md step 1).

use proptest::prelude::*;
use tt_core::rounding::round;
use tt_core::{RoundingDirection, RoundingLevel, RoundingRule, RoundingUnit};

const MIN: i64 = 60;

fn rule(unit: RoundingUnit, direction: RoundingDirection) -> RoundingRule {
    RoundingRule {
        unit,
        direction,
        level: RoundingLevel::Project,
    }
}

fn rounded(secs: i64, unit: RoundingUnit) -> [i64; 3] {
    [
        RoundingDirection::Up,
        RoundingDirection::Nearest,
        RoundingDirection::Down,
    ]
    .map(|d| round(secs, &rule(unit, d)))
}

#[test]
fn default_rule_is_15_minutes_nearest_project_level() {
    assert_eq!(
        RoundingRule::default(),
        rule(RoundingUnit::M15, RoundingDirection::Nearest)
    );
}

#[test]
fn worked_example_22_minutes_at_15() {
    // [up, nearest, down]
    assert_eq!(
        rounded(22 * MIN, RoundingUnit::M15),
        [30 * MIN, 15 * MIN, 15 * MIN]
    );
}

#[test]
fn exactly_half_rounds_up_for_nearest() {
    let nearest = rule(RoundingUnit::M15, RoundingDirection::Nearest);
    assert_eq!(round(22 * MIN + 30, &nearest), 30 * MIN);
    assert_eq!(round(22 * MIN + 29, &nearest), 15 * MIN);
}

#[test]
fn zero_stays_zero() {
    for unit in RoundingUnit::ALL {
        assert_eq!(rounded(0, unit), [0, 0, 0], "unit {unit:?}");
    }
}

#[test]
fn exact_multiples_are_unchanged() {
    for unit in RoundingUnit::ALL {
        let t = 3 * unit.seconds();
        assert_eq!(rounded(t, unit), [t, t, t], "unit {unit:?}");
    }
}

#[test]
fn one_example_per_unit() {
    // [up, nearest, down]
    assert_eq!(
        rounded(10 * MIN, RoundingUnit::M6),
        [12 * MIN, 12 * MIN, 6 * MIN]
    );
    assert_eq!(
        rounded(10 * MIN, RoundingUnit::M12),
        [12 * MIN, 12 * MIN, 0]
    );
    assert_eq!(
        rounded(44 * MIN, RoundingUnit::M30),
        [60 * MIN, 30 * MIN, 30 * MIN]
    );
    assert_eq!(
        rounded(45 * MIN, RoundingUnit::M30),
        [60 * MIN, 60 * MIN, 30 * MIN]
    );
    assert_eq!(
        rounded(60 * MIN + 1, RoundingUnit::M60),
        [120 * MIN, 60 * MIN, 60 * MIN]
    );
    assert_eq!(
        rounded(90 * MIN, RoundingUnit::M60),
        [120 * MIN, 120 * MIN, 60 * MIN]
    );
}

#[test]
fn only_exact_units_exist() {
    let minutes: Vec<u32> = RoundingUnit::ALL.iter().map(|u| u.minutes()).collect();
    assert_eq!(minutes, [6, 12, 15, 30, 60]);
    for m in 0..=120 {
        assert_eq!(
            RoundingUnit::from_minutes(m).is_some(),
            minutes.contains(&m),
            "{m}"
        );
    }
}

#[test]
fn rules_serialise_as_plain_values() {
    let json = serde_json::to_string(&RoundingRule::default()).unwrap();
    assert_eq!(
        json,
        r#"{"unit":15,"direction":"nearest","level":"project"}"#
    );
    let bad = r#"{"unit":10,"direction":"nearest","level":"project"}"#;
    assert!(serde_json::from_str::<RoundingRule>(bad).is_err());
}

#[test]
fn database_texts_round_trip() {
    for d in [
        RoundingDirection::Up,
        RoundingDirection::Nearest,
        RoundingDirection::Down,
    ] {
        assert_eq!(RoundingDirection::parse(d.as_str()), Some(d));
    }
    for l in [RoundingLevel::Project, RoundingLevel::Client] {
        assert_eq!(RoundingLevel::parse(l.as_str()), Some(l));
    }
    assert_eq!(RoundingDirection::parse("Up"), None);
}

#[test]
fn bundled_tzdb_has_berlin() {
    let tz = tt_core::testing::berlin();
    assert_eq!(tz.iana_name(), Some("Europe/Berlin"));
}

fn any_rule() -> impl Strategy<Value = RoundingRule> {
    (
        proptest::sample::select(RoundingUnit::ALL.to_vec()),
        proptest::sample::select(vec![
            RoundingDirection::Up,
            RoundingDirection::Nearest,
            RoundingDirection::Down,
        ]),
    )
        .prop_map(|(unit, direction)| rule(unit, direction))
}

/// Up to ten years of seconds: far more than any day total.
const MAX_SECS: i64 = 10 * 366 * 24 * 3600;

proptest! {
    #[test]
    fn rounding_is_idempotent(t in 0..=MAX_SECS, r in any_rule()) {
        let once = round(t, &r);
        prop_assert_eq!(round(once, &r), once);
    }

    #[test]
    fn result_is_a_multiple_of_the_unit(t in 0..=MAX_SECS, r in any_rule()) {
        prop_assert_eq!(round(t, &r) % r.unit.seconds(), 0);
    }

    #[test]
    fn down_le_raw_le_up(t in 0..=MAX_SECS, unit in proptest::sample::select(RoundingUnit::ALL.to_vec())) {
        let [up, nearest, down] = rounded(t, unit);
        prop_assert!(down <= t && t <= up);
        prop_assert!(down <= nearest && nearest <= up);
        prop_assert!(up - down == 0 || up - down == unit.seconds());
    }

    #[test]
    fn nearest_is_within_half_a_unit(t in 0..=MAX_SECS, unit in proptest::sample::select(RoundingUnit::ALL.to_vec())) {
        let nearest = round(t, &rule(unit, RoundingDirection::Nearest));
        prop_assert!((nearest - t).abs() <= unit.seconds() / 2);
    }
}
