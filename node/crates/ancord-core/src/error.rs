//! Chyby čistého ověřování (profil, stráž, `push_id`). Node je převádí na `DataError`.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProofError {
    #[error("{0}")]
    BadRequest(&'static str),
    #[error("invalid signature or proof of work")]
    InvalidProof,
}
