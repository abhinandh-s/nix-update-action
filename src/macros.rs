#[macro_export]
macro_rules! define_system {
    ( $( $contains:literal => $result:literal ),* $(,)? ) => {
        #[inline]
        pub fn system_matcher(s: &str) -> &'static str {
            match s {
                $(
                    n if n.contains($contains) => $result,
                )*
                    _ => panic!("No matching `system` found in `{s}`"),
            }
        }
    };
}


