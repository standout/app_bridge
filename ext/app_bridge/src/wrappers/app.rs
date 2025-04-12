use magnus::{Error, TryConvert, Value};
use std::cell::RefCell;
use wasmtime::Store;

use crate::app_state::AppState;
use crate::component::standout::app::types::{TriggerContext, TriggerResponse, AppError, ErrorCode};
use crate::component::{app, build_engine, build_linker, build_store, Bridge};
use super::trigger_context::RTriggerContext;

use super::trigger_response::RTriggerResponse;

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
    pub fn initialize(&self, component_path: String) {
        let mut this = self.0.borrow_mut();
        let engine = build_engine();
        let linker = build_linker(&engine).unwrap();
        let mut store = build_store(&engine);

        let app = app(component_path.clone(), engine, &mut store, linker).unwrap();

        this.component_path = component_path.to_string();
        *this.instance.borrow_mut() = Some(app);
        *this.store.borrow_mut() = Some(store);
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
                      Err(AppError {
                          code: ErrorCode::InternalError,
                          message: format!("Unexpected error: {:?}", err),
                      }.into())
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
}
