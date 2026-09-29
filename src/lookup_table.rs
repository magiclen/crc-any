/// This enum hold lookup table for static know or dynamic created table
#[derive(Clone)]
pub(crate) enum LookUpTable<T: 'static> {
    Static(&'static [T]),
    Dynamic([T; 256]),
}

impl<T> core::ops::Deref for LookUpTable<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        match *self {
            LookUpTable::Static(s) => s,
            LookUpTable::Dynamic(ref d) => d,
        }
    }
}
