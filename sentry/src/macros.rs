/// validates ParseError Eq implementation
macro_rules! validate {
	($cond:expr, $e:expr) => {
		if !($cond) {
			return Err($e);
		}
	};
	($cond:expr, $fmt:expr, $($arg:tt)+) => {
		if !($cond) {
			return Err($fmt, $($arg)+);
		}
	};
}
