// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {DisputeRegistry} from "../contracts/DisputeRegistry.sol";

contract DisputeRegistryTest is Test {
    DisputeRegistry internal registry;
    address internal governance = address(0x1001);
    address internal verifier = address(0x2002);
    address internal opener = address(0x3003);
    address internal outsider = address(0x4004);

    bytes32 internal disputeId = keccak256("dsp_demo_charlie_001");
    bytes32 internal commitmentId = keccak256("cmt_demo_charlie_001");
    bytes32 internal evidenceRoot = keccak256("evidence_root");
    bytes32 internal resolutionRoot = keccak256("resolution_root");
    uint64 internal deadline = uint64(block.timestamp + 2 days);

    event DisputeOpened(
        bytes32 indexed disputeId,
        bytes32 indexed commitmentId,
        address indexed openedBy,
        bytes32 evidenceRoot
    );

    event DisputeResolved(
        bytes32 indexed disputeId,
        DisputeRegistry.Resolution resolution,
        bytes32 resolutionEvidenceRoot
    );

    function setUp() public {
        registry = new DisputeRegistry(governance);
    }

    function _open() internal {
        vm.prank(opener);
        registry.openDispute(disputeId, commitmentId, evidenceRoot, deadline);
    }

    function testOpenDisputeStoresAndEmits() public {
        vm.expectEmit(true, true, true, true);
        emit DisputeOpened(disputeId, commitmentId, opener, evidenceRoot);
        _open();

        (
            bytes32 storedCommitment,
            address openedBy,
            bytes32 storedRoot,
            uint64 storedDeadline,
            DisputeRegistry.Resolution resolution,
            ,
            bool resolved
        ) = registry.getDispute(disputeId);

        assertEq(storedCommitment, commitmentId);
        assertEq(openedBy, opener);
        assertEq(storedRoot, evidenceRoot);
        assertEq(storedDeadline, deadline);
        assertEq(uint8(resolution), uint8(DisputeRegistry.Resolution.None));
        assertEq(resolved, false);
    }

    function testDuplicateOpenReverts() public {
        _open();
        vm.prank(opener);
        vm.expectRevert("dispute exists");
        registry.openDispute(disputeId, commitmentId, evidenceRoot, deadline);
    }

    function testOpenRequiresEvidenceRoot() public {
        vm.prank(opener);
        vm.expectRevert("evidence required");
        registry.openDispute(disputeId, commitmentId, bytes32(0), deadline);
    }

    function testResolveByGovernanceEmitsAndSets() public {
        _open();
        vm.expectEmit(true, false, false, true);
        emit DisputeResolved(disputeId, DisputeRegistry.Resolution.Refunded, resolutionRoot);
        vm.prank(governance);
        registry.resolveDispute(disputeId, DisputeRegistry.Resolution.Refunded, resolutionRoot);

        (, , , , DisputeRegistry.Resolution resolution, bytes32 storedRoot, bool resolved) =
            registry.getDispute(disputeId);
        assertEq(uint8(resolution), uint8(DisputeRegistry.Resolution.Refunded));
        assertEq(storedRoot, resolutionRoot);
        assertEq(resolved, true);
    }

    function testResolveByAssignedVerifier() public {
        _open();
        vm.prank(governance);
        registry.assignVerifier(disputeId, verifier);

        vm.prank(verifier);
        registry.resolveDispute(disputeId, DisputeRegistry.Resolution.Failed, resolutionRoot);

        (, , , , DisputeRegistry.Resolution resolution, , bool resolved) =
            registry.getDispute(disputeId);
        assertEq(uint8(resolution), uint8(DisputeRegistry.Resolution.Failed));
        assertEq(resolved, true);
    }

    function testResolveByUnauthorizedReverts() public {
        _open();
        vm.prank(outsider);
        vm.expectRevert("not authorized");
        registry.resolveDispute(disputeId, DisputeRegistry.Resolution.Refunded, resolutionRoot);
    }

    function testResolveTwiceReverts() public {
        _open();
        vm.prank(governance);
        registry.resolveDispute(disputeId, DisputeRegistry.Resolution.Refunded, resolutionRoot);

        vm.prank(governance);
        vm.expectRevert("already resolved");
        registry.resolveDispute(disputeId, DisputeRegistry.Resolution.Fulfilled, resolutionRoot);
    }

    function testResolveWithNoneReverts() public {
        _open();
        vm.prank(governance);
        vm.expectRevert("invalid resolution");
        registry.resolveDispute(disputeId, DisputeRegistry.Resolution.None, resolutionRoot);
    }

    function testResolveUnknownDisputeReverts() public {
        vm.prank(governance);
        vm.expectRevert("unknown dispute");
        registry.resolveDispute(keccak256("missing"), DisputeRegistry.Resolution.Refunded, resolutionRoot);
    }
}
