// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {ProviderRegistry} from "../contracts/ProviderRegistry.sol";

contract ProviderRegistryTest is Test {
    ProviderRegistry internal registry;
    address internal governance = address(0x1001);
    address internal registrant = address(0x2002);
    address internal outsider = address(0x3003);

    function setUp() public {
        registry = new ProviderRegistry(governance);
    }

    function testRegisterParticipantSetsRequested() public {
        vm.prank(registrant);
        registry.registerParticipant(
            "provider.example.travel",
            "provider",
            "sha256:manifest",
            500
        );

        (string memory state, address owner, string memory participantType, , uint256 stake, ) =
            registry.getParticipant("provider.example.travel");

        assertEq(state, "requested");
        assertEq(owner, registrant);
        assertEq(participantType, "provider");
        assertEq(stake, 500);
    }

    function testOnlyGovernanceCanTransition() public {
        vm.prank(registrant);
        registry.registerParticipant(
            "provider.example.travel",
            "provider",
            "sha256:manifest",
            500
        );

        vm.prank(outsider);
        vm.expectRevert("not governance");
        registry.transitionParticipant(
            "provider.example.travel",
            uint8(ProviderRegistry.State.IdentityVerified),
            "unauthorized"
        );
    }

    function testValidTransitionsReachActive() public {
        vm.prank(registrant);
        registry.registerParticipant(
            "provider.example.travel",
            "provider",
            "sha256:manifest",
            500
        );

        vm.startPrank(governance);
        registry.transitionParticipant(
            "provider.example.travel",
            uint8(ProviderRegistry.State.IdentityVerified),
            "identity ok"
        );
        registry.transitionParticipant(
            "provider.example.travel",
            uint8(ProviderRegistry.State.ConformancePassed),
            "conformance ok"
        );
        registry.transitionParticipant(
            "provider.example.travel",
            uint8(ProviderRegistry.State.Probation),
            "probation"
        );
        registry.transitionParticipant(
            "provider.example.travel",
            uint8(ProviderRegistry.State.Active),
            "promoted"
        );
        vm.stopPrank();

        (string memory state, , , , , ) = registry.getParticipant("provider.example.travel");
        assertEq(state, "active");
    }

    function testInvalidTransitionReverts() public {
        vm.prank(registrant);
        registry.registerParticipant(
            "provider.example.travel",
            "provider",
            "sha256:manifest",
            500
        );

        vm.prank(governance);
        vm.expectRevert("invalid transition");
        registry.transitionParticipant(
            "provider.example.travel",
            uint8(ProviderRegistry.State.Active),
            "skip states"
        );
    }
}
