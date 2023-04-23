pub mod instructions;
pub mod limit_order_instructions;
pub mod tests_suite;
pub mod utils;

#[tokio::test]
pub async fn test_limit_order_integration() {
    tests_suite::basic_interactions().await;
}
