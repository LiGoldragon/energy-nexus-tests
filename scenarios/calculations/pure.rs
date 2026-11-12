use energy_nexus_calculations::*;
use meta_signal_energy_nexus::library::{RedeemExpiring, RedeemUsedAtLeast};
use signal_energy_nexus::library::*;

fn reset(id: &str, expiry: Expiry) -> Reset {
    Reset {
        reset_id: ResetId(id.into()),
        reset_status: ResetStatus::Available,
        reset_kind: ResetKind::RateLimits,
        expiry,
        granted: Granted::Unknown,
    }
}
fn selection(resets: Vec<Reset>) -> CreditSelection {
    CreditSelection {
        credits: Credits::Known(Known_Data {
            available_count: AvailableCount(resets.len() as i64),
            reset_details: ResetDetails::Complete(resets),
        }),
        allowance: Allowance::Allowed,
        instant: Instant(1000),
        redeem_expiring: RedeemExpiring(10),
        redeem_used_at_least: RedeemUsedAtLeast(37),
        week_usage: WeekUsage::Percent(Percent(37)),
    }
}

#[test]
fn configured_rule_boundaries_are_inclusive_and_not_hardwired() {
    for (expiry, used, expected) in [(1600, 37, true), (1601, 37, false), (1600, 36, false)] {
        let candidate = reset("credit", Expiry::Instant(Instant(expiry)));
        let mut value = selection(vec![candidate.clone()]);
        value.week_usage = WeekUsage::Percent(Percent(used));
        assert_eq!(
            value.select(),
            if expected {
                SelectionResult::Reset(candidate)
            } else {
                SelectionResult::RuleRefused
            }
        );
    }
}
#[test]
fn denial_is_sufficient_with_an_explicit_waiting_work_request() {
    let candidate = reset("no-expiry", Expiry::NoExpiry);
    let mut value = selection(vec![candidate.clone()]);
    value.allowance = Allowance::Disallowed;
    value.week_usage = WeekUsage::Unknown;
    assert_eq!(value.select(), SelectionResult::Reset(candidate));
}
#[test]
fn no_expiry_is_not_an_expiring_deadline() {
    assert_eq!(
        selection(vec![reset("unlimited", Expiry::NoExpiry)]).select(),
        SelectionResult::RuleRefused
    );
}
#[test]
fn expiry_proof_can_satisfy_the_rule_with_unknown_allowance() {
    let candidate = reset("soon", Expiry::Instant(Instant(1600)));
    let mut value = selection(vec![candidate.clone()]);
    value.allowance = Allowance::Unknown;
    assert_eq!(value.select(), SelectionResult::Reset(candidate));
}
#[test]
fn unknown_weekly_use_does_not_become_zero() {
    let mut value = selection(vec![reset("soon", Expiry::Instant(Instant(1600)))]);
    value.redeem_used_at_least = RedeemUsedAtLeast(0);
    value.week_usage = WeekUsage::Unknown;
    assert_eq!(value.select(), SelectionResult::RuleRefused);
}
#[test]
fn finite_expiry_precedes_no_expiry_and_id_breaks_equal_ties() {
    let expected = reset("a", Expiry::Instant(Instant(1500)));
    let mut value = selection(vec![
        reset("unlimited", Expiry::NoExpiry),
        reset("b", Expiry::Instant(Instant(1500))),
        expected.clone(),
    ]);
    assert_eq!(value.select(), SelectionResult::Reset(expected.clone()));
    if let Credits::Known(known) = &mut value.credits {
        if let ResetDetails::Complete(resets) = &mut known.reset_details {
            resets.reverse();
        }
    }
    assert_eq!(value.select(), SelectionResult::Reset(expected));
}
#[test]
fn no_expiry_ties_are_deterministic() {
    let a = reset("a", Expiry::NoExpiry);
    let mut value = selection(vec![reset("z", Expiry::NoExpiry), a.clone()]);
    value.allowance = Allowance::Disallowed;
    assert_eq!(value.select(), SelectionResult::Reset(a));
}
#[test]
fn expired_credits_are_ineligible() {
    let candidate = reset("future", Expiry::Instant(Instant(1001)));
    let value = selection(vec![
        reset("past", Expiry::Instant(Instant(999))),
        reset("now", Expiry::Instant(Instant(1000))),
        candidate.clone(),
    ]);
    assert_eq!(value.select(), SelectionResult::Reset(candidate));
}
#[test]
fn unknown_expiry_prevents_choosing_a_later_credit() {
    let value = selection(vec![
        reset("unknown", Expiry::Unknown),
        reset("known", Expiry::Instant(Instant(1500))),
    ]);
    assert_eq!(value.select(), SelectionResult::Unproved);
}
#[test]
fn incomplete_or_inconsistent_credit_evidence_cannot_prove_ordering() {
    for details in [
        ResetDetails::Unknown,
        ResetDetails::Partial(vec![]),
        ResetDetails::Complete(vec![]),
    ] {
        let mut value = selection(vec![]);
        value.credits = Credits::Known(Known_Data {
            available_count: AvailableCount(1),
            reset_details: details,
        });
        assert_eq!(value.select(), SelectionResult::Unproved);
    }
}
#[test]
fn zero_summary_proves_no_reset_without_inventing_details() {
    let mut value = selection(vec![]);
    value.credits = Credits::Known(Known_Data {
        available_count: AvailableCount(0),
        reset_details: ResetDetails::Unknown,
    });
    assert_eq!(value.select(), SelectionResult::NoReset);
}
#[test]
fn partial_details_never_prove_no_reset_from_a_zero_summary() {
    for listed in [
        vec![],
        vec![reset("listed", Expiry::Instant(Instant(1500)))],
    ] {
        for allowance in [
            Allowance::Allowed,
            Allowance::Disallowed,
            Allowance::Unknown,
        ] {
            let mut value = selection(vec![]);
            value.allowance = allowance;
            value.credits = Credits::Known(Known_Data {
                available_count: AvailableCount(0),
                reset_details: ResetDetails::Partial(listed.clone()),
            });
            assert_eq!(value.select(), SelectionResult::Unproved);
        }
    }
}
#[test]
fn duplicate_identity_or_unknown_status_or_kind_prevents_selection() {
    let known = reset("same", Expiry::Instant(Instant(1500)));
    assert_eq!(
        selection(vec![known.clone(), known.clone()]).select(),
        SelectionResult::Unproved
    );
    let mut unknown_status = known.clone();
    unknown_status.reset_status = ResetStatus::Unknown;
    assert_eq!(
        selection(vec![unknown_status]).select(),
        SelectionResult::Unproved
    );
    let mut unknown_kind = known;
    unknown_kind.reset_kind = ResetKind::Unknown;
    assert_eq!(
        selection(vec![unknown_kind]).select(),
        SelectionResult::Unproved
    );
}
#[test]
fn invalid_policy_is_refused_and_large_policy_arithmetic_is_safe() {
    let mut value = selection(vec![reset("soon", Expiry::Instant(Instant(i64::MAX)))]);
    value.redeem_expiring = RedeemExpiring(i64::MAX);
    assert!(matches!(value.select(), SelectionResult::Reset(_)));
    value.redeem_expiring = RedeemExpiring(0);
    assert_eq!(value.select(), SelectionResult::RuleRefused);
    value.redeem_expiring = RedeemExpiring(10);
    value.redeem_used_at_least = RedeemUsedAtLeast(101);
    assert_eq!(value.select(), SelectionResult::RuleRefused);
}

fn window(used: i64) -> Window {
    Window {
        name: Name::Week,
        usage: Usage::Percent(Percent(used)),
        reset_time: ResetTime::Instant(Instant(86400)),
        window_duration: WindowDuration::Minutes(Minutes(1440)),
    }
}
fn level(windows: Windows) -> LevelCalculation {
    LevelCalculation {
        reading: Reading {
            provider: Provider::Codex,
            allowance: Allowance::Allowed,
            windows,
            credits: Credits::Unknown,
            at: At(43200),
        },
        threshold_vector: vec![
            Threshold {
                level: Level::Plenty,
                left: Left(50),
            },
            Threshold {
                level: Level::Fair,
                left: Left(20),
            },
            Threshold {
                level: Level::Low,
                left: Left(0),
            },
        ],
    }
}
#[test]
fn explicit_denial_proves_out_even_when_windows_are_unknown() {
    let mut value = level(Windows::MissingEvidence(MissingEvidence::NotProvided));
    value.reading.allowance = Allowance::Disallowed;
    assert_eq!(value.level(), Level::Out);
}
#[test]
fn missing_partial_empty_and_unknown_usage_never_prove_plenty() {
    let mut missing_usage = window(0);
    missing_usage.usage = Usage::Unknown;
    for windows in [
        Windows::Known(vec![]),
        Windows::Known(vec![missing_usage]),
        Windows::MissingEvidence(MissingEvidence::NotProvided),
        Windows::Partial(Partial_Data {
            window_vector: vec![window(0)],
            missing_evidence: MissingEvidence::ScopeIdentityUnverified,
        }),
    ] {
        assert_eq!(level(windows).level(), Level::Unknown);
    }
}
#[test]
fn unknown_allowance_and_unverified_window_identity_remain_unknown() {
    let mut value = level(Windows::Known(vec![window(10)]));
    value.reading.allowance = Allowance::Unknown;
    assert_eq!(value.level(), Level::Unknown);
    let mut unverified = window(10);
    unverified.name = Name::Unknown;
    assert_eq!(
        level(Windows::Known(vec![unverified])).level(),
        Level::Unknown
    );
}
#[test]
fn level_uses_most_used_window_and_supplied_thresholds() {
    let mut second = window(79);
    second.name = Name::Session;
    let mut value = level(Windows::Known(vec![window(10), second]));
    assert_eq!(value.level(), Level::Fair);
    value.threshold_vector[1].left = Left(22);
    assert_eq!(value.level(), Level::Low);
}
#[test]
fn duplicate_scope_and_out_of_range_use_are_unknown() {
    assert_eq!(
        level(Windows::Known(vec![window(0), window(0)])).level(),
        Level::Unknown
    );
    assert_eq!(
        level(Windows::Known(vec![window(101)])).level(),
        Level::Unknown
    );
}
#[test]
fn ambiguous_threshold_boundaries_are_unknown_in_every_permutation() {
    let thresholds = [
        Threshold {
            level: Level::Plenty,
            left: Left(20),
        },
        Threshold {
            level: Level::Fair,
            left: Left(20),
        },
        Threshold {
            level: Level::Low,
            left: Left(0),
        },
    ];
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        for used in [0, 80, 100] {
            let mut value = level(Windows::Known(vec![window(used)]));
            value.threshold_vector = order
                .iter()
                .map(|&index| thresholds[index].clone())
                .collect();
            assert_eq!(value.level(), Level::Unknown);
        }
    }
}
#[test]
fn repeated_identical_thresholds_preserve_the_level_in_every_position() {
    let original = level(Windows::Known(vec![window(79)]));
    for index in 0..=original.threshold_vector.len() {
        let mut value = original.clone();
        value
            .threshold_vector
            .insert(index, value.threshold_vector[1].clone());
        assert_eq!(value.level(), Level::Fair);
    }
}

fn pace() -> PaceCalculation {
    PaceCalculation {
        provider: Provider::Codex,
        window: window(25),
        target: Target::Even,
        instant: Instant(43200),
    }
}
#[test]
fn half_a_day_remaining_requires_one_hundred_fifty_points_per_day() {
    assert_eq!(
        pace().pace(),
        Pace::Calculated(Calculated_Data {
            provider: Provider::Codex,
            name: Name::Week,
            target: Target::Even,
            expected: Expected(50),
            used: Used(25),
            standing: Standing::Under,
            per_day: PerDay(150)
        })
    );
}
#[test]
fn one_second_remaining_preserves_the_rate_above_one_hundred() {
    let mut value = pace();
    value.instant = Instant(86399);
    value.window.usage = Usage::Percent(Percent(99));
    let Pace::Calculated(result) = value.pace() else {
        panic!("known input");
    };
    assert_eq!(result.per_day, PerDay(86400));
    assert_eq!(result.expected, Expected(100));
}
#[test]
fn spend_by_and_finished_deadlines_preserve_existing_numerical_policy() {
    let mut value = pace();
    value.instant = Instant(86400);
    value.window.usage = Usage::Percent(Percent(75));
    value.target = Target::SpendBy(Instant(172800));
    let Pace::Calculated(result) = value.pace() else {
        panic!("known input");
    };
    assert_eq!(
        (result.expected, result.per_day, result.standing),
        (Expected(50), PerDay(25), Standing::Over)
    );
    value.target = Target::Even;
    let Pace::Calculated(result) = value.pace() else {
        panic!("known input");
    };
    assert_eq!(result.per_day, PerDay(0));
    assert_eq!(result.expected, Expected(100));
}
#[test]
fn missing_pace_inputs_have_their_own_typed_reason() {
    for (index, reason) in [
        (0, PaceUnavailable::IdentityMissing),
        (1, PaceUnavailable::UsageMissing),
        (2, PaceUnavailable::ResetTimeMissing),
        (3, PaceUnavailable::DurationMissing),
    ] {
        let mut value = pace();
        match index {
            0 => value.window.name = Name::Unknown,
            1 => value.window.usage = Usage::Unknown,
            2 => value.window.reset_time = ResetTime::Unknown,
            _ => value.window.window_duration = WindowDuration::Unknown,
        }
        assert_eq!(
            value.pace(),
            Pace::Unavailable(Unavailable_Data {
                provider: Provider::Codex,
                name: value.window.name,
                pace_unavailable: reason
            })
        );
    }
}
#[test]
fn large_valid_timestamps_and_durations_do_not_overflow() {
    let mut value = pace();
    value.window.window_duration = WindowDuration::Minutes(Minutes(i64::MAX));
    value.window.reset_time = ResetTime::Instant(Instant(i64::MAX));
    value.instant = Instant(i64::MAX - 1);
    value.window.usage = Usage::Percent(Percent(99));
    let Pace::Calculated(result) = value.pace() else {
        panic!("known input");
    };
    assert_eq!(result.per_day, PerDay(86400));
}
#[test]
fn generated_selection_retains_imported_payloads() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/selection.rs"));
    assert!(source.contains("Percent(signal_energy_nexus::library::Percent),"));
    assert!(source.contains("Reset(signal_energy_nexus::library::Reset),"));
}
