use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub enum FileType<T> {
    Symlink(T),
    Directory(T),
}

impl<T> FileType<T>
where
    T: AsRef<Path>,
{
    pub fn path(&self) -> &Path {
        match self {
            Self::Symlink(p) => p.as_ref(),
            Self::Directory(p) => p.as_ref(),
        }
    }
}

impl<T, O> PartialEq<FileType<O>> for FileType<T>
where
    T: PartialEq<O>,
{
    fn eq(&self, other: &FileType<O>) -> bool {
        match (self, other) {
            (FileType::Symlink(s), FileType::Symlink(o)) => s == o,
            (FileType::Directory(s), FileType::Directory(o)) => s == o,
            _ => false,
        }
    }
}

impl<T> Eq for FileType<T> where T: Eq {}
impl<T> Copy for FileType<T> where T: Copy {}

pub type FilePath<'a> = FileType<&'a Path>;
pub type FilePathBuf = FileType<PathBuf>;

#[allow(dead_code)]
impl<'a> FilePath<'a> {
    pub fn new_symlink<T>(path: &'a T) -> Self
    where
        T: 'a + AsRef<Path> + ?Sized,
    {
        Self::Symlink(path.as_ref())
    }
    pub fn new_directory<T>(path: &'a T) -> Self
    where
        T: 'a + AsRef<Path> + ?Sized,
    {
        Self::Directory(path.as_ref())
    }
    pub fn replace_path<T>(self, path: &'a T) -> Self
    where
        T: 'a + AsRef<Path> + ?Sized,
    {
        match self {
            Self::Symlink(_) => Self::Symlink(path.as_ref()),
            Self::Directory(_) => Self::Directory(path.as_ref()),
        }
    }
}

#[allow(dead_code)]
impl FilePathBuf {
    pub fn new_symlink(path: impl Into<PathBuf>) -> Self {
        Self::Symlink(path.into())
    }
    pub fn new_directory(path: impl Into<PathBuf>) -> Self {
        Self::Directory(path.into())
    }
    pub fn replace_path(self, path: impl Into<PathBuf>) -> Self {
        match self {
            Self::Symlink(_) => Self::Symlink(path.into()),
            Self::Directory(_) => Self::Directory(path.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;

    macro_rules! new_tests {
        ($test_name:ident, $ctor:ident, $subtype:ident) => {
            #[test]
            fn $test_name() {
                // FilePath
                assert_eq!(
                    FilePath::$ctor("test"),
                    FilePath::$subtype(Path::new("test")),
                );
                assert_eq!(
                    FilePath::$ctor(Path::new("test")),
                    FilePath::$subtype(Path::new("test")),
                );
                // FilePathBuf
                assert_eq!(
                    FilePathBuf::$ctor("test"),
                    FilePathBuf::$subtype("test".into()),
                );
                assert_eq!(
                    FilePathBuf::$ctor(Path::new("test")),
                    FilePathBuf::$subtype("test".into()),
                );
                assert_eq!(
                    FilePathBuf::$ctor(PathBuf::from("test")),
                    FilePathBuf::$subtype("test".into()),
                );
            }
        };
    }

    macro_rules! replace_tests {
        ($test_name:ident, $ctor:ident, $subtype:ident) => {
            #[test]
            fn $test_name() {
                // FilePath
                assert_eq!(
                    FilePath::$ctor("bad").replace_path("test"),
                    FilePath::$subtype(Path::new("test")),
                );
                assert_eq!(
                    FilePath::$ctor("bad").replace_path(Path::new("test")),
                    FilePath::$subtype(Path::new("test")),
                );
                // FilePathBuf
                assert_eq!(
                    FilePathBuf::$ctor("bad").replace_path("test"),
                    FilePathBuf::$subtype("test".into()),
                );
                assert_eq!(
                    FilePathBuf::$ctor("bad").replace_path(Path::new("test")),
                    FilePathBuf::$subtype("test".into()),
                );
                assert_eq!(
                    FilePathBuf::$ctor("bad").replace_path(PathBuf::from("test")),
                    FilePathBuf::$subtype("test".into()),
                );
            }
        };
    }

    new_tests!(test_file_type_new_symlink, new_symlink, Symlink);
    new_tests!(test_file_type_new_directory, new_directory, Directory);
    replace_tests!(test_file_type_symlink_replace, new_symlink, Symlink);
    replace_tests!(test_file_type_directory_replace, new_directory, Directory);

    #[test]
    fn test_file_path_debug() {
        let f = FilePath::Symlink(Path::new("test"));
        assert_eq!(format!("{f:?}"), r#"Symlink("test")"#);
    }

    #[test]
    fn test_file_path_buf_debug() {
        let f = FilePathBuf::Symlink(PathBuf::from("test"));
        assert_eq!(format!("{f:?}"), r#"Symlink("test")"#);
    }

    #[test]
    fn test_file_type_partial_eq() {
        // FilePath.
        assert_eq!(FilePath::new_symlink("test"), FilePath::new_symlink("test"));
        assert_eq!(
            FilePath::new_directory("test"),
            FilePath::new_directory("test"),
        );
        assert_ne!(
            FilePath::new_symlink("test"),
            FilePath::new_directory("test"),
        );
        // FilePathBuf.
        assert_eq!(
            FilePathBuf::new_symlink("test"),
            FilePathBuf::new_symlink("test"),
        );
        // Mixed types.
        assert_eq!(
            FilePath::new_symlink("test"),
            FilePathBuf::new_symlink("test"),
        );
        assert_eq!(
            FilePath::new_directory("test"),
            FilePathBuf::new_directory("test"),
        );
        assert_eq!(
            FilePathBuf::new_symlink("test"),
            FilePath::new_symlink("test"),
        );
        assert_eq!(
            FilePathBuf::new_directory("test"),
            FilePath::new_directory("test"),
        );
        assert_ne!(
            FilePath::new_symlink("test"),
            FilePathBuf::new_directory("test"),
        );
    }
}
