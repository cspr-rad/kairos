use mimicaw::{Args, Test as MCTest, Outcome};
use std::future::Future;
use std::pin::Pin;
use tokio::runtime::Runtime;

mod tests;
pub mod fixtures;

pub use crate::tests::{Test, Outcome as E2EResult};

pub fn run_tests() {
    let args = Args::from_env().unwrap_or_else(|e| e.exit());

    let tests: Vec<mimicaw::Test<fn() -> Pin<Box<dyn Future<Output = Outcome>>>>> =
        inventory::iter::<crate::Test>()
            .map(|test| MCTest::test(test.name, test.test_fn))
            .collect::<Vec<_>>();

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        mimicaw::run_tests(&args, tests, |_, boxed_future: fn() -> Pin<Box<dyn Future<Output = Outcome>>>| async move {
            let future = boxed_future();
            future.await
        })
        .await
        .exit();
    });
}