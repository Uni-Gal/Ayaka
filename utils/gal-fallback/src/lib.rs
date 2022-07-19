pub struct Fallback<T> {
    data: Option<T>,
    base_data: Option<T>,
}

impl<T> Fallback<T> {
    pub const fn new(data: Option<T>, base_data: Option<T>) -> Self {
        Self { data, base_data }
    }

    pub const fn is_some(&self) -> bool {
        self.data.is_some() || self.base_data.is_some()
    }

    pub const fn as_ref(&self) -> Fallback<&T> {
        Fallback::new(self.data.as_ref(), self.base_data.as_ref())
    }

    pub fn and_then<V>(self, mut f: impl FnMut(T) -> Option<V>) -> Option<V> {
        self.data
            .and_then(|t| f(t))
            .or_else(|| self.base_data.and_then(|t| f(t)))
    }

    pub fn map<V>(self, mut f: impl FnMut(T) -> V) -> Fallback<V> {
        Fallback::new(self.data.map(|t| f(t)), self.base_data.map(|t| f(t)))
    }

    pub fn unzip(self) -> (Option<T>, Option<T>) {
        (self.data, self.base_data)
    }
}

impl<T> Fallback<Option<T>> {
    pub fn flatten(self) -> Fallback<T> {
        Fallback::new(self.data.flatten(), self.base_data.flatten())
    }
}

impl Fallback<String> {
    pub fn and_any(self) -> Option<String> {
        self.and_then(|s| if s.is_empty() { None } else { Some(s) })
    }
}

impl<T> Fallback<Vec<T>> {
    pub fn and_any(self) -> Option<Vec<T>> {
        self.and_then(|s| if s.is_empty() { None } else { Some(s) })
    }
}

impl<T> From<Fallback<T>> for Option<T> {
    fn from(f: Fallback<T>) -> Self {
        if f.data.is_some() {
            f.data
        } else {
            f.base_data
        }
    }
}

pub struct FallbackIter<A> {
    data: A,
    base_data: A,
}

impl<A: Iterator> Iterator for FallbackIter<A> {
    type Item = Fallback<A::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let d = self.data.next();
        let based = self.base_data.next();
        if d.is_some() || based.is_some() {
            Some(Fallback::new(d, based))
        } else {
            None
        }
    }
}

impl<T> IntoIterator for Fallback<Vec<T>> {
    type Item = Fallback<T>;

    type IntoIter = FallbackIter<<Vec<T> as IntoIterator>::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        FallbackIter {
            data: self.data.unwrap_or_default().into_iter(),
            base_data: self.base_data.unwrap_or_default().into_iter(),
        }
    }
}

pub trait FallbackSpec: Sized {
    type SpecType: From<Fallback<Self>>;
}

impl<T: FallbackSpec> Fallback<T> {
    pub fn spec(self) -> T::SpecType {
        T::SpecType::from(self)
    }
}

pub use gal_fallback_derive::FallbackSpec;

#[cfg(test)]
mod test {
    mod gal_fallback {
        pub use crate::*;
    }
    use gal_fallback::*;

    #[test]
    fn some() {
        assert!(!Fallback::<()>::new(None, None).is_some());
    }

    #[test]
    fn option() {
        let f = Fallback::new(None, Some(100));
        assert_eq!(Option::from(f), Some(100));
    }
}
