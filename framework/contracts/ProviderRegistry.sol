// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

/// @title ProviderRegistry
/// @notice Reference registry contract for Covenant Network participant onboarding.
/// @dev This contract intentionally keeps a minimal state model and explicit transitions.
contract ProviderRegistry {
    enum State {
        None,
        Requested,
        IdentityVerified,
        ConformancePassed,
        Probation,
        Active,
        Restricted,
        Revoked
    }

    struct Participant {
        string participantId;
        address owner;
        string participantType;
        string manifestHash;
        uint256 stake;
        State state;
        uint256 updatedAt;
        bool exists;
    }

    address public governance;
    mapping(bytes32 => Participant) private participants;

    event ParticipantRegistered(
        string indexed participantId,
        address indexed owner,
        string participantType,
        string manifestHash,
        uint256 stake
    );

    event ParticipantTransitioned(
        string indexed participantId,
        uint8 fromState,
        uint8 toState,
        string reason
    );

    event ParticipantAttested(
        string indexed participantId,
        string attestationType,
        bytes attestationPayload
    );

    event GovernanceChanged(address indexed oldGovernance, address indexed newGovernance);

    modifier onlyGovernance() {
        require(msg.sender == governance, "not governance");
        _;
    }

    constructor(address governanceAddress) {
        require(governanceAddress != address(0), "invalid governance");
        governance = governanceAddress;
    }

    function setGovernance(address newGovernance) external onlyGovernance {
        require(newGovernance != address(0), "invalid governance");
        address old = governance;
        governance = newGovernance;
        emit GovernanceChanged(old, newGovernance);
    }

    function registerParticipant(
        string calldata participantId,
        string calldata participantType,
        string calldata manifestHash,
        uint256 stake
    ) external returns (bytes32 registrationId) {
        bytes32 id = keccak256(bytes(participantId));
        require(!participants[id].exists, "already registered");
        require(bytes(participantId).length > 0, "participantId required");
        require(bytes(participantType).length > 0, "participantType required");
        require(bytes(manifestHash).length > 0, "manifestHash required");

        participants[id] = Participant({
            participantId: participantId,
            owner: msg.sender,
            participantType: participantType,
            manifestHash: manifestHash,
            stake: stake,
            state: State.Requested,
            updatedAt: block.timestamp,
            exists: true
        });

        emit ParticipantRegistered(participantId, msg.sender, participantType, manifestHash, stake);
        registrationId = id;
    }

    function transitionParticipant(
        string calldata participantId,
        uint8 toState,
        string calldata reason
    ) external onlyGovernance {
        bytes32 id = keccak256(bytes(participantId));
        Participant storage p = participants[id];
        require(p.exists, "unknown participant");
        require(toState <= uint8(State.Revoked), "invalid state");

        State from = p.state;
        _requireValidTransition(uint8(from), toState);

        p.state = State(toState);
        p.updatedAt = block.timestamp;
        emit ParticipantTransitioned(participantId, uint8(from), toState, reason);
    }

    function addAttestation(
        string calldata participantId,
        string calldata attestationType,
        bytes calldata attestationPayload
    ) external onlyGovernance {
        bytes32 id = keccak256(bytes(participantId));
        require(participants[id].exists, "unknown participant");
        emit ParticipantAttested(participantId, attestationType, attestationPayload);
    }

    function getParticipant(
        string calldata participantId
    )
        external
        view
        returns (
            string memory state,
            address owner,
            string memory participantType,
            string memory manifestHash,
            uint256 stake,
            uint256 updatedAt
        )
    {
        bytes32 id = keccak256(bytes(participantId));
        Participant storage p = participants[id];
        require(p.exists, "unknown participant");
        return (_stateToString(p.state), p.owner, p.participantType, p.manifestHash, p.stake, p.updatedAt);
    }

    function _requireValidTransition(uint8 from, uint8 to) internal pure {
        // requested -> identity_verified
        if (from == uint8(State.Requested) && to == uint8(State.IdentityVerified)) return;
        // identity_verified -> conformance_passed
        if (from == uint8(State.IdentityVerified) && to == uint8(State.ConformancePassed)) return;
        // conformance_passed -> probation
        if (from == uint8(State.ConformancePassed) && to == uint8(State.Probation)) return;
        // probation -> active|restricted
        if (from == uint8(State.Probation) && (to == uint8(State.Active) || to == uint8(State.Restricted))) return;
        // active -> restricted
        if (from == uint8(State.Active) && to == uint8(State.Restricted)) return;
        // restricted -> probation|revoked
        if (from == uint8(State.Restricted) && (to == uint8(State.Probation) || to == uint8(State.Revoked))) return;
        revert("invalid transition");
    }

    function _stateToString(State s) internal pure returns (string memory) {
        if (s == State.Requested) return "requested";
        if (s == State.IdentityVerified) return "identity_verified";
        if (s == State.ConformancePassed) return "conformance_passed";
        if (s == State.Probation) return "probation";
        if (s == State.Active) return "active";
        if (s == State.Restricted) return "restricted";
        if (s == State.Revoked) return "revoked";
        return "none";
    }
}
