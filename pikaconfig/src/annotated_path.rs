use std::path::{Path, PathBuf};

// AnnotatedPath represents a path with a custom debug representation.
pub trait AnnotatedPath: std::fmt::Debug {
    fn path(&self) -> &Path;
}

pub type AnnotatedPathBox = Box<dyn AnnotatedPath>;

impl AnnotatedPath for PathBuf {
    fn path(&self) -> &Path {
        self
    }
}

impl AnnotatedPath for &Path {
    fn path(&self) -> &Path {
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
        assert_eq!(ap.path(), path);
    }

    #[test]
    fn test_path() {
        let path = Path::new("test");
        let ap: AnnotatedPathBox = Box::new(path);
        assert_eq!(format!("{ap:?}"), "\"test\"");
        assert_eq!(ap.path(), path);
    }
}
