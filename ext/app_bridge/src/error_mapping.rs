use crate::types::{AppError, ErrorCode};
use magnus::prelude::*;
use magnus::{Error, ExceptionClass, RObject, Ruby};

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
            ErrorCode::RetryWithReference(_) => get_class("AppBridge::RetryWithReferenceError"),
            ErrorCode::CompleteWorkflow => get_class("AppBridge::CompleteWorkflowException"),
            ErrorCode::CompleteParent => get_class("AppBridge::CompleteParentException"),
        }
    }
}

impl From<AppError> for Error {
    fn from(value: AppError) -> Self {
        if let ErrorCode::RetryWithReference(retry) = value.code.clone() {
            let class: ExceptionClass = value.code.into();
            let message = value.message;
            if let Ok(exception) = class.new_instance((message.as_str(),)) {
                if let Ok(exception_value) = RObject::try_convert(exception.as_value()) {
                    let _ = exception_value.ivar_set("@reference", retry.reference);
                    let _ = exception_value.ivar_set("@status", retry.status);
                }
                return Error::from(exception);
            }

            return Error::new(class, message);
        }

        Error::new(value.code.into(), value.message)
    }
}
