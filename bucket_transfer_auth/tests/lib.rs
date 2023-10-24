use std::env;

use scrypto_unit::*;
use transaction::prelude::*;
use dot_random_test_utils::{deploy_random_component};
use radix_engine::transaction::CommitResult;


#[test]
fn test_request_mint_with_bucket() {
    // dir is different in Debug mode
    let root_dir = env::current_dir().ok().unwrap().ends_with("dot-random-examples");
    let dir_example = if root_dir { "./bucket_transfer_auth" } else { "../bucket_transfer_auth" };
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    // Deploy RandomComponent
    let mut random_env = deploy_random_component(&mut test_runner, "59ae807");

    // Deploy ExampleCaller
    let package_address2 = test_runner.publish_retain_blueprints(
        dir_example,
        |blueprint, _| blueprint.eq("ExampleCaller"),
    );
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address2,
                "ExampleCaller",
                "instantiate",
                manifest_args!(),
            )
            .build(), vec![]);

    let result = receipt.expect_commit_success();
    let example_component = result.new_component_addresses()[0];
    let kv_store = get_kv_store(result);

    // Act
    // 1. Request mint - should return callback id: 1
    let receipt = test_runner.execute_manifest_ignoring_fee(
        ManifestBuilder::new()
            .call_method(
                example_component,
                "request_mint",
                manifest_args!(),
            )
            .build(), vec![]);
    let result = receipt.expect_commit_success();
    let out = result.outcome.expect_success();
    out[1].expect_return_value(&1u32);

    // 2. Simulate a TX that calls RandomComponent.execute() to do the actual mint - should mint an NFT
    random_env.execute_next(&mut test_runner, 1);

    // Assert
    let minted_nft: Option<u32> = test_runner.get_kv_store_entry(Own(*kv_store), &1u16);
    assert_eq!(Some(0x270C0794), minted_nft);
}

fn get_kv_store(result: &CommitResult) -> &NodeId {
    // Our KVS is last created.
    let mut last: &NodeId = &NodeId([1u8; 30]);
    for (node_id, _) in &result.state_updates.by_node {
        if node_id.is_internal_kv_store() {
            last = node_id;
        }
    }
    return last;
}