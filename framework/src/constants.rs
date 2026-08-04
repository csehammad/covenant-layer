pub const REQUIRED_STATES: &[&str] = &[
    "requested",
    "identity_verified",
    "conformance_passed",
    "probation",
    "active",
    "restricted",
    "revoked",
];

pub fn allowed_targets(from: &str) -> &'static [&'static str] {
    match from {
        "requested" => &["identity_verified"],
        "identity_verified" => &["conformance_passed"],
        "conformance_passed" => &["probation"],
        "probation" => &["active", "restricted"],
        "active" => &["restricted"],
        "restricted" => &["probation", "revoked"],
        "revoked" => &[],
        _ => &[],
    }
}

/// Settlement states a commitment may be in for a dispute to be opened.
///
/// Reason codes and resolution decisions live in [`crate::domain`] as typed
/// enums; disputable states remain a string set because settlement state is a
/// broader vocabulary shared with the (stringly-typed) commitment record.
pub const DISPUTABLE_SETTLEMENT_STATES: &[&str] = &["fulfilled", "failed", "settled"];

pub fn is_disputable_state(state: &str) -> bool {
    DISPUTABLE_SETTLEMENT_STATES.contains(&state)
}
