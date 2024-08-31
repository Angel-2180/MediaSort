use std::fmt::{self, Display, Formatter};


/// Custom error type for early exit.
#[derive(Debug)]
pub struct SilentExit {
    #[allow(dead_code)]
    pub code: u8,
}

impl Display for SilentExit {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
