use std::io;

pub(crate) trait IsNotFound {
    fn is_not_found(&self) -> bool;
}

impl IsNotFound for io::Error {
    fn is_not_found(&self) -> bool {
        self.kind() == io::ErrorKind::NotFound
    }
}

impl IsNotFound for anyhow::Error {
    fn is_not_found(&self) -> bool {
        match self.downcast_ref::<io::Error>() {
            Some(err) => err.is_not_found(),
            None => false,
        }
    }
}

pub(crate) fn is_not_found(err: &impl IsNotFound) -> bool {
    err.is_not_found()
}

/// Returns Ok(None) on std::io::ErrorKind::NotFound, result otherwise.
pub(crate) fn skip_not_found<T, E>(result: Result<T, E>) -> Result<Option<T>, E>
where
    E: IsNotFound,
{
    match result {
        Ok(t) => Ok(Some(t)),
        Err(err) => {
            if is_not_found(&err) {
                Ok(None)
            } else {
                Err(err)
            }
        }
    }
}
