use thiserror::Error;
pub use rust_socketio::Error as SocketioErrorKind;
pub use serde_json::Error as JsonError;
pub use super::lwnsim_cmd::CmdErrorKind;


pub(crate) type Result<T> = std::result::Result<T, Error>;


#[derive(Error, Debug)]
pub enum Error{
    #[error("Cmd error : {0}")]
   CmdError(CmdErrorKind),
   #[error("Socketio error : {0}")]
   SocketioError(#[from] SocketioErrorKind),
   #[error("Json error : {0}")]
   InvalidJson(#[from] JsonError),
}






/* impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        Self::InvalidPoisonedLock()
    }
}

impl From<Error> for std::io::Error {
    fn from(err: Error) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, err)
    }
}
 */


