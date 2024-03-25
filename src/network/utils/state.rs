#[macro_export]
macro_rules! network_state {
    ($($tt:tt)+) => {
        #[derive(Debug, Default)]
        enum State { #[default] $($tt)+ }

        macro_rules! validate_state {
            ($conn:ident, $state:ident == $check:pat, $reason:literal) => {
                if !matches!($state, $check) {
                    $crate::network_disconnect!(&mut $conn, $reason);
                }
            };
        }
    };
}
