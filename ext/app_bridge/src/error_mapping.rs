use crate::component::standout::app::types::{AppError, ErrorCode};
use magnus::{self, Error, Ruby, ExceptionClass};

impl From<ErrorCode> for ExceptionClass {
  fn from(value: ErrorCode) -> Self {
    fn get_class(name: &str) -> ExceptionClass {
        let ruby = Ruby::get().unwrap();
        ruby.eval::<ExceptionClass>(name).unwrap()
    }

    match value {
        ErrorCode::Unauthenticated => get_class("AppBridge::UnauthenticatedError"),
        ErrorCode::Forbidden => get_class("AppBridge::ForbiddenError"),
        ErrorCode::Misconfigured => get_class("AppBridge::MisconfiguredError"),
        ErrorCode::Unsupported => get_class("AppBridge::UnsupportedError"),
        ErrorCode::RateLimit => get_class("AppBridge::RateLimitError"),
        ErrorCode::Timeout => get_class("AppBridge::TimeoutError"),
        ErrorCode::Unavailable => get_class("AppBridge::UnavailableError"),
        ErrorCode::InternalError => get_class("AppBridge::InternalError"),
        ErrorCode::MalformedResponse => get_class("AppBridge::MalformedResponseError"),
        ErrorCode::Other => get_class("AppBridge::OtherError"),
    }
  }
}

impl From<AppError> for Error {
  fn from(value: AppError) -> Self {
      Error::new(value.code.into(), value.message)
  }
}
