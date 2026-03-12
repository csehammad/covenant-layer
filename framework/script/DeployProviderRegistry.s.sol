// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {Script} from "forge-std/Script.sol";
import {ProviderRegistry} from "../contracts/ProviderRegistry.sol";

contract DeployProviderRegistry is Script {
    function run() external returns (ProviderRegistry registry) {
        address governance = vm.envAddress("GOVERNANCE_ADDRESS");
        vm.startBroadcast();
        registry = new ProviderRegistry(governance);
        vm.stopBroadcast();
    }
}
