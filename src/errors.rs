#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum LibraryError {
    #[error("IO Error: {0}")]
    #[diagnostic(help("Check if the file exists and is accessible."))]
    IO(#[from] std::io::Error),

    #[error("Reader Error: {0}")]
    #[diagnostic(help("The data could not be read correctly."))]
    Reader(#[from] crate::errors::ReaderError),
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum ReaderError {
    #[error("Parse Error: {0}")]
    #[diagnostic(help("The data could not be parsed correctly."))]
    ParseError(&'static str),
}
