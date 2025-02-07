
#[macro_export]
macro_rules! println {
    () => (
        // impl
    );
    ($($arg:tt)*) => ({
        // impl
        core::format_args_nl!($($arg)*);
        
    })
}
pub mod error {
    pub use core_error::Error;
}
pub mod option {
    pub use core::option::*;
}
pub mod fmt {
    pub use core::fmt::*;
}
pub mod convert {
    pub use core::convert::*;
}
pub mod string {
    pub use alloc::string::*;
}
pub mod prelude {
    pub use core::prelude::rust_2021::*;
    pub use super::string::*;
    pub use alloc::boxed::Box;
    pub use alloc::vec;
    pub use alloc::vec::Vec;
    pub use alloc::fmt::Debug;
    pub use alloc::borrow::ToOwned;
    pub use alloc::format;
    #[cfg(not(feature = "std"))]
    pub use crate::println;

}
