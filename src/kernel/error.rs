//! 
//! fn foo() -> Result<String, Box<dyn std::error::Error>> {
//!     Err(Box::new(StrError("Error...")))
//! }
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
///
/// Error container
#[derive(Debug)]
pub struct StrError(String);
// Error doesn't require you to implement any methods, but
// your type must also implement Debug and Display.
impl std::error::Error for StrError {}
//
//
impl std::fmt::Display for StrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Delegate to the Display impl for `&str`:
        self.0.fmt(f)
    }
}
impl From<String> for StrError {
    fn from(value: String) -> Self {
        StrError(value)
    }
}
impl From<&str> for StrError {
    fn from(value: &str) -> Self {
        StrError(value.to_owned())
    }
}
impl Into<Box<StrError>> for &str {
    fn into(self) -> Box<StrError> {
        Box::new(StrError(self.to_owned()))
    }
}