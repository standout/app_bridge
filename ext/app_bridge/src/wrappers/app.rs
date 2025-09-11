use magnus::{Error, TryConvert, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use wasmtime::Store;

use crate::app_state::AppState;
use crate::component::standout::app::types::{
    AppError, ErrorCode, TriggerContext, TriggerResponse, ActionContext, ActionResponse
};
use crate::component::{app, build_engine, build_linker, build_store, Bridge};
use super::{
    trigger_context::RTriggerContext,
    trigger_response::RTriggerResponse,
    action_context::RActionContext,
    action_response::RActionResponse,
};

#[derive(Default)]
pub struct RApp {
    component_path: String,
    instance: RefCell<Option<Bridge>>,
    store: RefCell<Option<Store<AppState>>>,
}

#[derive(Default)]
#[magnus::wrap(class = "AppBridge::App")]
pub struct MutRApp(RefCell<RApp>);

impl MutRApp {
    pub fn initialize(&self, component_path: String, env_vars: HashMap<String, String>) -> Result<(), Error> {
        let mut this = self.0.borrow_mut();
        let engine = build_engine();
        let linker = build_linker(&engine).map_err(|e| {
            Error::new(
                magnus::exception::runtime_error(),
                format!("Failed to build linker: {}", e)
            )
        })?;
        let mut store = if env_vars.is_empty() {
            build_store(&engine, None)
        } else {
            build_store(&engine, Some(env_vars))
        };

        let app = app(component_path.clone(), engine, &mut store, linker).map_err(|e| {
            if e.to_string().contains("Incompatible WASM file version") {
                Error::new(
                    magnus::exception::runtime_error(),
                    e.to_string()
                )
            } else {
                Error::new(
                    magnus::exception::runtime_error(),
                    format!("Failed to initialize app: {}", e)
                )
            }
        })?;

        this.component_path = component_path.to_string();
        *this.instance.borrow_mut() = Some(app);
        *this.store.borrow_mut() = Some(store);

        Ok(())
    }

    pub fn trigger_ids(&self) -> Result<Vec<String>, Error> {
        let binding = self.0.borrow();

        let mut instance = binding.instance.borrow_mut();
        let mut store = binding.store.borrow_mut();

        if let (Some(instance), Some(store)) = (&mut *instance, &mut *store) {
            match instance.standout_app_triggers().call_trigger_ids(store) {
                Ok(result) => {
                    match result {
                        Ok(ids) => Ok(ids),
                        Err(err) => Err(err.into())
                    }
                },
                Err(err) => {
                  if let Some(wit_err) = err.downcast_ref::<AppError>() {
                      Err(wit_err.clone().into())
                  } else {
                      Err(Error::new(
                          magnus::exception::runtime_error(),
                          format!("Unexpected error: {:?}", err)
                      ))
                  }
                },
            }
        } else {
            Err(AppError {
                code: ErrorCode::InternalError,
                message: "App instance couln't be initialized".to_string(),
            }.into())
        }
    }

    pub fn trigger_input_schema(&self, context: RTriggerContext) -> Result<String, Error> {
        let binding = self.0.borrow();

        let mut instance = binding.instance.borrow_mut();
        let mut store = binding.store.borrow_mut();

        if let (Some(instance), Some(store)) = (&mut *instance, &mut *store) {
            let context_ctx = context.into();
            match instance.standout_app_triggers().call_input_schema(store, &context_ctx) {
                Ok(result) => {
                    match result {
                        Ok(schema) => Ok(schema),
                        Err(err) => Err(err.into())
                    }
                },
                Err(err) => {
                  if let Some(wit_err) = err.downcast_ref::<AppError>() {
                      Err(wit_err.clone().into())
                  } else {
                      Err(Error::new(
                          magnus::exception::runtime_error(),
                          format!("Unexpected error: {:?}", err)
                      ))
                  }
                },
            }
        } else {
            Err(AppError {
                code: ErrorCode::InternalError,
                message: "App instance couln't be initialized".to_string(),
            }.into())
        }
    }

    pub fn trigger_output_schema(&self, context: RTriggerContext) -> Result<String, Error> {
        let binding = self.0.borrow();

        let mut instance = binding.instance.borrow_mut();
        let mut store = binding.store.borrow_mut();

        if let (Some(instance), Some(store)) = (&mut *instance, &mut *store) {
            let context_ctx = context.into();
            match instance.standout_app_triggers().call_output_schema(store, &context_ctx) {
                Ok(result) => {
                    match result {
                        Ok(schema) => Ok(schema),
                        Err(err) => Err(err.into())
                    }
                },
                Err(err) => {
                  if let Some(wit_err) = err.downcast_ref::<AppError>() {
                      Err(wit_err.clone().into())
                  } else {
                      Err(Error::new(
                          magnus::exception::runtime_error(),
                          format!("Unexpected error: {:?}", err)
                      ))
                  }
                },
            }
        } else {
            Err(AppError {
                code: ErrorCode::InternalError,
                message: "App instance couln't be initialized".to_string(),
            }.into())
        }
    }

    pub fn rb_fetch_events(&self, context: Value) -> Result<RTriggerResponse, magnus::Error> {
        let context: RTriggerContext = TryConvert::try_convert(context).unwrap();
        let response = self.fetch_events(context.into());

        match response {
            Ok(response) =>  Ok(response.into()),
            Err(err) => Err(err.into())
        }
    }

    fn fetch_events(&self, context: TriggerContext) -> Result<TriggerResponse, AppError> {
        let binding = self.0.borrow();

        let mut instance = binding.instance.borrow_mut();
        let mut store = binding.store.borrow_mut();

        if let (Some(instance), Some(store)) = (&mut *instance, &mut *store) {
            match instance
                .standout_app_triggers()
                .call_fetch_events(store, &context) {
                Ok(response) => {
                    match response {
                        Ok(res) => Ok(res),
                        Err(err) => Err(err),
                    }
                },
                Err(err) => {
                    if let Some(wit_err) = err.downcast_ref::<AppError>() {
                        Err(AppError::from(wit_err.clone()))
                    } else {
                        Err(AppError {
                            code: ErrorCode::InternalError,
                            message: format!("Unexpected error: {:?}", err),
                        })
                    }
                }
            }
        } else {
            Err(AppError {
                code: ErrorCode::InternalError,
                message: "App instance couln't be initialized".to_string(),
            })
        }
    }

    pub fn action_ids(&self) -> Result<Vec<String>, Error> {
        let binding = self.0.borrow();

        let mut instance = binding.instance.borrow_mut();
        let mut store = binding.store.borrow_mut();

        if let (Some(instance), Some(store)) = (&mut *instance, &mut *store) {
            match instance.standout_app_actions().call_action_ids(store) {
                Ok(result) => {
                    match result {
                        Ok(ids) => Ok(ids),
                        Err(err) => Err(err.into())
                    }
                },
                Err(err) => {
                  if let Some(wit_err) = err.downcast_ref::<AppError>() {
                      Err(wit_err.clone().into())
                  } else {
                      Err(Error::new(
                          magnus::exception::runtime_error(),
                          format!("Unexpected error: {:?}", err)
                      ))
                  }
                },
            }
        } else {
            Err(AppError {
                code: ErrorCode::InternalError,
                message: "App instance couln't be initialized".to_string(),
            }.into())
        }
    }

    pub fn action_input_schema(&self, context: RActionContext) -> Result<String, Error> {
        let binding = self.0.borrow();

        let mut instance = binding.instance.borrow_mut();
        let mut store = binding.store.borrow_mut();

        if let (Some(instance), Some(store)) = (&mut *instance, &mut *store) {
            let context_ctx = context.into();
            match instance.standout_app_actions().call_input_schema(store, &context_ctx) {
                Ok(result) => {
                    match result {
                        Ok(schema) => Ok(schema),
                        Err(err) => Err(err.into())
                    }
                },
                Err(err) => {
                  if let Some(wit_err) = err.downcast_ref::<AppError>() {
                      Err(wit_err.clone().into())
                  } else {
                      Err(Error::new(
                          magnus::exception::runtime_error(),
                          format!("Unexpected error: {:?}", err)
                      ))
                  }
                },
            }
        } else {
            Err(AppError {
                code: ErrorCode::InternalError,
                message: "App instance couln't be initialized".to_string(),
            }.into())
        }
    }

    pub fn action_output_schema(&self, context: RActionContext) -> Result<String, Error> {
        let binding = self.0.borrow();

        let mut instance = binding.instance.borrow_mut();
        let mut store = binding.store.borrow_mut();

        if let (Some(instance), Some(store)) = (&mut *instance, &mut *store) {
            let context_ctx = context.into();
            match instance.standout_app_actions().call_output_schema(store, &context_ctx) {
                Ok(result) => {
                    match result {
                        Ok(schema) => Ok(schema),
                        Err(err) => Err(err.into())
                    }
                },
                Err(err) => {
                  if let Some(wit_err) = err.downcast_ref::<AppError>() {
                      Err(wit_err.clone().into())
                  } else {
                      Err(Error::new(
                          magnus::exception::runtime_error(),
                          format!("Unexpected error: {:?}", err)
                      ))
                  }
                },
            }
        } else {
            Err(AppError {
                code: ErrorCode::InternalError,
                message: "App instance couln't be initialized".to_string(),
            }.into())
        }
    }

    pub fn rb_execute_action(&self, context: Value) -> Result<RActionResponse, magnus::Error> {
        let context: RActionContext = TryConvert::try_convert(context).unwrap();
        let response = self.execute_action(context.into());

        match response {
            Ok(response) =>  Ok(response.into()),
            Err(err) => Err(err.into())
        }
    }

    fn execute_action(&self, context: ActionContext) -> Result<ActionResponse, AppError> {
        let binding = self.0.borrow();

        let mut instance = binding.instance.borrow_mut();
        let mut store = binding.store.borrow_mut();

        if let (Some(instance), Some(store)) = (&mut *instance, &mut *store) {
            match instance
                .standout_app_actions()
                .call_execute(store, &context) {
                Ok(response) => {
                    match response {
                        Ok(res) => Ok(res),
                        Err(err) => Err(err),
                    }
                },
                Err(err) => {
                    if let Some(wit_err) = err.downcast_ref::<AppError>() {
                        Err(AppError::from(wit_err.clone()))
                    } else {
                        Err(AppError {
                            code: ErrorCode::InternalError,
                            message: format!("Unexpected error: {:?}", err),
                        })
                    }
                }
            }
        } else {
            Err(AppError {
                code: ErrorCode::InternalError,
                message: "App instance couln't be initialized".to_string(),
            })
        }
    }
}
