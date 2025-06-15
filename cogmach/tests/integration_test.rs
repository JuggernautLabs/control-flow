use ::cogmach::*;
use client_implementations::client::MockVoid;

#[tokio::test]
async fn test_cognition_machine_creation() {
    let client = MockVoid;
    let _cogmach = FundamentalCognitionMachine::new(client);
    // Basic test to ensure the machine can be created
}

#[tokio::test]
async fn test_fundamental_experiment() {
    // Test that the experiment function runs without panic
    let result = run_fundamental_experiment().await;
    assert!(result.is_ok());
}
