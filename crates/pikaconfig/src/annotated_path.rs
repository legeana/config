use std::path::{Path, PathBuf};

// AnnotatedPath represents a path with a custom debug representation.
pub(crate) trait AnnotatedPath: std::fmt::Debug {
    fn as_path(&self) -> &Path;
    fn to_path_buf(&self) -> PathBuf {
        self.as_path().to_path_buf()
    }
}

pub(crate) type BoxedAnnotatedPath = Box<dyn AnnotatedPath>;

impl<T: AnnotatedPath + ?Sized> AnnotatedPath for &T {
    fn as_path(&self) -> &Path {
        T::as_path(self)
    }
    fn to_path_buf(&self) -> PathBuf {
        T::to_path_buf(self)
    }
}

impl<T: AnnotatedPath + ?Sized> AnnotatedPath for Box<T> {
    fn as_path(&self) -> &Path {
        T::as_path(self)
    }
    fn to_path_buf(&self) -> PathBuf {
        T::to_path_buf(self)
    }
}

impl AnnotatedPath for PathBuf {
    fn as_path(&self) -> &Path {
        self
    }
}

impl AnnotatedPath for Path {
    fn as_path(&self) -> &Path {
        self
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_path_buf() {
        let path = Path::new("test");
        let ap: BoxedAnnotatedPath = Box::new(path.to_owned());
        assert_eq!(format!("{ap:?}"), "\"test\"");
        assert_eq!(ap.as_path(), path);
        assert_eq!(ap.to_path_buf(), path);
    }

    #[test]
    fn test_path() {
        let path = Path::new("test");
        let ap: BoxedAnnotatedPath = Box::new(path);
        assert_eq!(format!("{ap:?}"), "\"test\"");
        assert_eq!(ap.as_path(), path);
        assert_eq!(ap.to_path_buf(), path);
    }
}
