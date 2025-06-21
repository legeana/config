#[macro_export]
macro_rules! assert_matches {
    ($e:expr, $($tail:tt)*) => {
        {
            let expr = $e;
            assert!(
                matches!(expr, $($tail)*),
                "{} doesn't match {}: got {expr:?}",
                stringify!($e),
                stringify!($($tail)*),
            );
        }
    };
}
