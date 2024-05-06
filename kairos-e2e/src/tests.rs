pub use mimicaw::Outcome;
use std::future::Future;
use std::pin::Pin;

pub struct Test {
    pub name: &'static str,
    pub test_fn: fn() -> Pin<Box<dyn Future<Output = Outcome>>>,
}

impl Test {
    pub const fn new(name: &'static str, test_fn: fn() -> Pin<Box<dyn Future<Output = Outcome>>>) -> Self {
        Self {
            name: name,
            test_fn,
        }
    }
}

#[macro_export]
macro_rules! register_test {
    ($name:expr, $func:path) => {
        inventory::submit! {
            Test::new($name, || std::boxed::Box::pin($func()))
        }
    };
}

inventory::collect!(Test);