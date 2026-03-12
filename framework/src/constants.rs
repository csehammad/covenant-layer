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
