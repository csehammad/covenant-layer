# Compatibility Matrix

This section defines how protocol versions, conformance profiles, and schema versions interoperate.

## Baseline matrix

| Protocol core | Wire/schema | Conformance profile | Status |
|---|---|---|---|
| `2.x` (`02-core-protocol.md`) | `1.0` (`05-wire-integrity.md`, `schemas/protocol-envelope.v1.schema.json`) | `onboarding-v1` | required baseline |

## Compatibility rules

- Patch/minor updates in `2.x` MUST remain compatible with wire/schema `1.0` and `onboarding-v1`.
- Breaking wire changes MUST increment wire/schema major version.
- Breaking onboarding requirements MUST increment conformance profile version.
- Implementations SHOULD advertise supported `(protocol, wire, profile)` tuples.

## Negotiation and failure behavior

When sender and receiver do not share a compatible tuple:

- receiver MUST reject with `ERR_PROFILE_UNSUPPORTED` or version-specific equivalent
- receiver SHOULD return supported tuples in response metadata
- sender SHOULD retry only with compatible versions

## Deprecation policy

- New major versions SHOULD provide at least one deprecation window before mandatory cutover.
- Deprecation windows SHOULD include migration guides and test vectors.
- Emergency security breaks MAY shorten deprecation windows with explicit governance notice.

## Migration checklist

- publish change rationale and security impact
- publish compatibility matrix update
- publish example objects for old and new versions
- run conformance on both versions during migration window
- publish retirement date for deprecated profile/version
