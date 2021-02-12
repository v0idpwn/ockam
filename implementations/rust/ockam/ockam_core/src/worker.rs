use crate::Result;

/// Base ockam worker trait.  See [`Handler`] for message receival
///
/// [`Handler`]: self::Handler
#[async_trait::async_trait]
pub trait Worker: Send + 'static {
    type Context: Send + 'static;

    /// Override initialisation behaviour
    fn initialize(&mut self, _context: &mut Self::Context) -> Result<()> {
        Ok(())
    }

    /// Override shutdown behaviour
    fn shutdown(&mut self, _context: &mut Self::Context) -> Result<()> {
        Ok(())
    }
}
