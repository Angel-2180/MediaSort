use std::fmt::{self, Display, Formatter};
use std::io;

use anyhow::{bail, Context, Result};

/// Custom error type for early exit.
#[derive(Debug)]
pub struct SilentExit {
    pub code: u8,
}

impl Display for SilentExit {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
