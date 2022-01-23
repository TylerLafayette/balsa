use crate::errors::BalsaError;

/// The [`Result`] type for all public Balsa API methods.
///
/// See [`BalsaError`] for error descriptions.
pub type BalsaResult<T> = Result<T, BalsaError>;
