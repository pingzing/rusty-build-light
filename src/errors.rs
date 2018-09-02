#[derive(Debug, Fail)]
pub enum UnityRetrievalError {
    #[fail(
        display = "Unity Cloud Build returned a response, but no build information was contained."
    )]
    NoBuildsReturned,

    #[fail(display = "Unity Cloud Build returned an HTTP error: {}", http_error_message)]
    HttpError { http_error_message: String },
}
