use alloc::boxed::Box;

use super::{Backtrace, ErrorCompat};

impl<E> ErrorCompat for Box<E>
where
    E: ErrorCompat,
{
    fn backtrace(&self) -> Option<&Backtrace> {
        (**self).backtrace()
    }
}
