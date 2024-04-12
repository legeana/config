use std::path::{Path, PathBuf};

// AnnotatedPath represents a path with a custom debug representation.
pub trait AnnotatedPath: std::fmt::Debug {
    fn as_path(&self) -> &Path;
}

pub type AnnotatedPathBox = Box<dyn AnnotatedPath>;

impl<T: AnnotatedPath + ?Sized> AnnotatedPath for &T {
    fn as_path(&self) -> &Path {
        T::as_path(self)
    }
}

impl<T: AnnotatedPath + ?Sized> AnnotatedPath for Box<T> {
    fn as_path(&self) -> &Path {
        T::as_path(self)
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
        let ap: AnnotatedPathBox = Box::new(path.to_owned());
        assert_eq!(format!("{ap:?}"), "\"test\"");
        assert_eq!(ap.as_path(), path);
    }

    #[test]
    fn test_path() {
        let path = Path::new("test");
        let ap: AnnotatedPathBox = Box::new(path);
        assert_eq!(format!("{ap:?}"), "\"test\"");
        assert_eq!(ap.as_path(), path);
    }
}
