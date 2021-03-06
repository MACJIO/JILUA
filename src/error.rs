use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecompileError {
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("Invalid uleb128 value was passed.")]
    InvalidULeb128,
    #[error("Invalid byte code header bytes")]
    InvalidHeaderBytes(&'static str),
    #[error("Unknown instruction opcode.")]
    UnknownInsOpcode,
    #[error("Unexpected instruction opcode.")]
    UnexpectedInsOpcode,
    #[error("Invalid primitive type value.")]
    InvalidPriValue,
}
