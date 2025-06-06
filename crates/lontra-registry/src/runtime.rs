#![allow(dead_code)]

use anyhow::Context as _;
use anyhow::Result;
use tokio::runtime::Builder;
use tokio::runtime::Runtime as TokioRuntime;

#[derive(Debug)]
pub(crate) struct Runtime(TokioRuntime);

impl Runtime {
    pub(crate) fn new_current_thread() -> Result<Self> {
        Ok(Self(
            Builder::new_current_thread()
                .enable_all()
                .build()
                .context("failed to build tokio::Runtime")?,
        ))
    }
    pub(crate) fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future,
    {
        self.0.block_on(future)
    }
}

#[cfg(test)]
pub(crate) fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    Runtime::new_current_thread()
        .expect("Runtime::new_current_thread")
        .block_on(future)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_block_on() {
        let mut x = 0;

        block_on(async {
            x = 10;
        });

        assert_eq!(x, 10);
    }
}
