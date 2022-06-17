#[derive(Debug, strum::EnumIter)]
pub enum ErrorKind {
    DBError(String),
    InvalidInput(String),
    InternalError(String),
    NotImplemented(String),
}
