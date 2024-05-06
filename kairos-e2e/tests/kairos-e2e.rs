use kairos_e2e::{register_test, Test, E2EResult};

async fn test_kairos_e2e() -> E2EResult {
    println!("This test was ran with kairos-e2e");
    E2EResult::passed()
}

register_test!("test_kairos_e2e", test_kairos_e2e);

fn main() {
    kairos_e2e::run_tests();
}