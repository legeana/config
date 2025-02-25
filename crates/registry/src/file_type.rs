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
    pub fn file_type(&self) -> Type {
        match self {
            Self::Symlink(_) => Type::Symlink(()),
            Self::Directory(_) => Type::Directory(()),
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

pub(crate) type Type = FileType<()>;
pub type FilePath<'a> = FileType<&'a Path>;
pub type FilePathBuf = FileType<PathBuf>;

impl Type {
    pub fn with_path(self, path: &Path) -> FilePath {
        match self {
            Self::Symlink(()) => FilePath::Symlink(path),
            Self::Directory(()) => FilePath::Directory(path),
        }
    }
    pub fn with_path_buf(self, path: PathBuf) -> FilePathBuf {
        match self {
            Self::Symlink(()) => FilePathBuf::Symlink(path),
            Self::Directory(()) => FilePathBuf::Directory(path),
        }
    }
}

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
}

impl FilePathBuf {
    pub fn new_symlink(path: impl Into<PathBuf>) -> Self {
        Self::Symlink(path.into())
    }
    pub fn new_directory(path: impl Into<PathBuf>) -> Self {
        Self::Directory(path.into())
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

    new_tests!(test_file_type_new_symlink, new_symlink, Symlink);
    new_tests!(test_file_type_new_directory, new_directory, Directory);

    #[test]
    fn file_type_file_type() {
        assert_eq!(FilePath::new_symlink("test").file_type(), Type::Symlink(()));
        assert_eq!(
            FilePath::new_directory("test").file_type(),
            Type::Directory(()),
        );
    }

    #[test]
    fn test_type_with_path() {
        assert_eq!(
            Type::Symlink(()).with_path(Path::new("test")),
            FilePath::Symlink(Path::new("test")),
        );
        assert_eq!(
            Type::Directory(()).with_path(Path::new("test")),
            FilePath::Directory(Path::new("test")),
        );
    }

    #[test]
    fn test_type_with_path_buf() {
        assert_eq!(
            Type::Symlink(()).with_path_buf(PathBuf::from("test")),
            FilePathBuf::Symlink(PathBuf::from("test")),
        );
        assert_eq!(
            Type::Directory(()).with_path_buf(PathBuf::from("test")),
            FilePathBuf::Directory(PathBuf::from("test")),
        );
    }

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
