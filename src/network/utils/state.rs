#[macro_export]
macro_rules! network_state {
    (phase => $phase:literal; $($tt:tt)+) => {
        #[derive(Debug, Default)]
        enum State { #[default] $($tt)+ }

        macro_rules! validate_state {
            ($conn:ident, $state:ident == $sty:tt::$check:tt, $reason:literal) => {
                if !matches!($state, $sty::$check) {
                    tracing::warn!(phase = $phase, expected = ?$sty::$check, got = ?$state, "State validation failed");
                    $crate::network_disconnect!(&mut $conn, $reason);
                }
            };
        }
    };
}
