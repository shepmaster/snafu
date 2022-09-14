/// An iterator over an Error and its sources.
///
/// If you want to omit the initial error and only process its sources, use `skip(1)`.
///
/// Can be created via [`ErrorCompat::iter_chain`][crate::ErrorCompat::iter_chain].
pub struct ChainCompat<'a> {
    inner: Option<&'a dyn std::error::Error>,
}

impl<'a> ChainCompat<'a> {
    /// Creates a new error chain iterator.
    pub fn new(error: &'a dyn std::error::Error) -> Self {
        ChainCompat { inner: Some(error) }
    }
}

impl<'a> Iterator for ChainCompat<'a> {
    type Item = &'a dyn std::error::Error;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner {
            None => None,
            Some(e) => {
                self.inner = e.source();
                Some(e)
            }
        }
    }
}
