# Provider Registry Contract Interface

This document defines the minimum EVM-facing contract interface expected by the framework.

## Functions

```solidity
function registerParticipant(
    string calldata participantId,
    string calldata participantType,
    string calldata manifestHash,
    uint256 stake
) external returns (bytes32 registrationId);

function transitionParticipant(
    string calldata participantId,
    string calldata toState,
    string calldata reason
) external;

function addAttestation(
    string calldata participantId,
    string calldata attestationType,
    bytes calldata attestationPayload
) external;

function getParticipant(string calldata participantId)
    external
    view
    returns (
        string memory state,
        address owner,
        string memory manifestHash,
        uint256 stake,
        uint256 updatedAt
    );
```

## Events

```solidity
event ParticipantRegistered(
    string indexed participantId,
    address indexed owner,
    string participantType,
    string manifestHash,
    uint256 stake
);

event ParticipantTransitioned(
    string indexed participantId,
    string fromState,
    string toState,
    string reason
);

event ParticipantAttested(
    string indexed participantId,
    string attestationType,
    bytes attestationPayload
);
```

## Ownership and control constraints

- Registration path should remain permissionless for valid submissions.
- Transition rights may be role-gated but must be transparent and auditable.
- Upgrades should be timelocked and governed by multisig/DAO policy.
- Governance should be multisig/DAO-controlled from initial deployment.
