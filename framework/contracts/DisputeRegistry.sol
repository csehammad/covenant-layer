// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

/// @title DisputeRegistry
/// @notice Minimal on-chain registry for Covenant dispute lifecycle anchoring.
/// @dev Only digests/roots and lifecycle transitions are stored on-chain; full
///      evidence payloads remain off-chain, consistent with Covenant's hybrid
///      design. This contract is intentionally scoped narrowly and does not
///      expand the participant onboarding registry.
contract DisputeRegistry {
    enum Resolution {
        None,
        Fulfilled,
        Failed,
        Refunded
    }

    struct Dispute {
        bytes32 commitmentId;
        address openedBy;
        bytes32 evidenceRoot;
        uint64 responseDeadline;
        Resolution resolution;
        bytes32 resolutionEvidenceRoot;
        bool exists;
        bool resolved;
    }

    /// @notice Settlement authority permitted to resolve any dispute.
    address public governance;

    mapping(bytes32 => Dispute) private disputes;

    /// @notice Optional per-dispute assigned verifier permitted to resolve it.
    mapping(bytes32 => address) public assignedVerifier;

    event DisputeOpened(
        bytes32 indexed disputeId,
        bytes32 indexed commitmentId,
        address indexed openedBy,
        bytes32 evidenceRoot
    );

    event DisputeResolved(
        bytes32 indexed disputeId,
        Resolution resolution,
        bytes32 resolutionEvidenceRoot
    );

    event VerifierAssigned(bytes32 indexed disputeId, address indexed verifier);

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

    /// @notice Assign a verifier authorized to resolve a specific dispute.
    function assignVerifier(bytes32 disputeId, address verifier) external onlyGovernance {
        require(verifier != address(0), "invalid verifier");
        assignedVerifier[disputeId] = verifier;
        emit VerifierAssigned(disputeId, verifier);
    }

    /// @notice Open a dispute. Any address may open; enforcement of commitment
    ///         party membership is performed off-chain by the framework, while
    ///         `openedBy` (msg.sender) is anchored here for auditability.
    function openDispute(
        bytes32 disputeId,
        bytes32 commitmentId,
        bytes32 evidenceRoot,
        uint64 responseDeadline
    ) external {
        require(disputeId != bytes32(0), "disputeId required");
        require(!disputes[disputeId].exists, "dispute exists");
        require(evidenceRoot != bytes32(0), "evidence required");

        disputes[disputeId] = Dispute({
            commitmentId: commitmentId,
            openedBy: msg.sender,
            evidenceRoot: evidenceRoot,
            responseDeadline: responseDeadline,
            resolution: Resolution.None,
            resolutionEvidenceRoot: bytes32(0),
            exists: true,
            resolved: false
        });

        emit DisputeOpened(disputeId, commitmentId, msg.sender, evidenceRoot);
    }

    /// @notice Resolve a dispute. Only the assigned verifier or the settlement
    ///         authority (governance) may resolve; a resolved dispute cannot be
    ///         re-resolved under the same identifier.
    function resolveDispute(
        bytes32 disputeId,
        Resolution resolution,
        bytes32 resolutionEvidenceRoot
    ) external {
        Dispute storage d = disputes[disputeId];
        require(d.exists, "unknown dispute");
        require(!d.resolved, "already resolved");
        require(resolution != Resolution.None, "invalid resolution");
        require(
            msg.sender == governance || msg.sender == assignedVerifier[disputeId],
            "not authorized"
        );

        d.resolution = resolution;
        d.resolutionEvidenceRoot = resolutionEvidenceRoot;
        d.resolved = true;

        emit DisputeResolved(disputeId, resolution, resolutionEvidenceRoot);
    }

    function getDispute(bytes32 disputeId)
        external
        view
        returns (
            bytes32 commitmentId,
            address openedBy,
            bytes32 evidenceRoot,
            uint64 responseDeadline,
            Resolution resolution,
            bytes32 resolutionEvidenceRoot,
            bool resolved
        )
    {
        Dispute storage d = disputes[disputeId];
        require(d.exists, "unknown dispute");
        return (
            d.commitmentId,
            d.openedBy,
            d.evidenceRoot,
            d.responseDeadline,
            d.resolution,
            d.resolutionEvidenceRoot,
            d.resolved
        );
    }
}
