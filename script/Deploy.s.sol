// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../contracts/SandwichExecutor.sol";

contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        
        vm.startBroadcast(deployerPrivateKey);
        
        SandwichExecutor executor = new SandwichExecutor();
        
        console.log("SandwichExecutor deployed to:", address(executor));
        console.log("Owner:", executor.owner());
        
        vm.stopBroadcast();
    }
}
