// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {Script} from "forge-std/Script.sol";
import {DisputeRegistry} from "../contracts/DisputeRegistry.sol";

contract DeployDisputeRegistry is Script {
    function run() external returns (DisputeRegistry registry) {
        address governance = vm.envAddress("GOVERNANCE_ADDRESS");
        vm.startBroadcast();
        registry = new DisputeRegistry(governance);
        vm.stopBroadcast();
    }
}
