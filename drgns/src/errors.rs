use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("hmm")]
pub struct DragonError {
    #[source_code]
    src: crate::arena::SpanOld,
}
