use crate::app_state::AppState;
use crate::component::{v3, v4};
use crate::component::v4::standout::app::http::{Method, Request, RequestError, Response};
use reqwest::Method as ReqwestMethod;
use std::result::Result::Ok;
use wasmtime::component::Resource;

// ============================================================================
// Macro to implement HostRequestBuilder for any version
//
// When adding a new version, just add:
//   impl_host_request_builder!(v5);
//   impl_http_type_conversions!(v5);
// ============================================================================

macro_rules! impl_host_request_builder {
    ($v:ident) => {
        impl $v::standout::app::http::HostRequestBuilder for AppState {
            fn new(&mut self) -> Resource<$v::standout::app::http::RequestBuilder> {
                let id = self.next_request_id;
                self.next_request_id += 1;
                self.request_list.insert(id, Request::default());
                Resource::new_own(id)
            }

            fn method(
                &mut self,
                self_: Resource<$v::standout::app::http::RequestBuilder>,
                method: $v::standout::app::http::Method,
            ) -> Resource<$v::standout::app::http::RequestBuilder> {
                let id = self_.rep();
                if let Some(mut request) = self.request_list.get(&id).cloned() {
                    request.method = method.into();
                    let new_id = self.next_request_id;
                    self.next_request_id += 1;
                    self.request_list.insert(new_id, request);
                    Resource::new_own(new_id)
                } else {
                    Resource::new_own(id)
                }
            }

            fn url(
                &mut self,
                self_: Resource<$v::standout::app::http::RequestBuilder>,
                url: String,
            ) -> Resource<$v::standout::app::http::RequestBuilder> {
                let id = self_.rep();
                if let Some(mut request) = self.request_list.get(&id).cloned() {
                    request.url = url;
                    let new_id = self.next_request_id;
                    self.next_request_id += 1;
                    self.request_list.insert(new_id, request);
                    Resource::new_own(new_id)
                } else {
                    Resource::new_own(id)
                }
            }

            fn header(
                &mut self,
                self_: Resource<$v::standout::app::http::RequestBuilder>,
                key: String,
                value: String,
            ) -> Resource<$v::standout::app::http::RequestBuilder> {
                let id = self_.rep();
                let mut request = self.request_list.get(&id).cloned().unwrap_or_default();
                request.headers.push((key, value));
                let new_id = self.next_request_id;
                self.next_request_id += 1;
                self.request_list.insert(new_id, request);
                Resource::new_own(new_id)
            }

            fn headers(
                &mut self,
                self_: Resource<$v::standout::app::http::RequestBuilder>,
                headers: Vec<(String, String)>,
            ) -> Resource<$v::standout::app::http::RequestBuilder> {
                let id = self_.rep();
                let mut request = self.request_list.get(&id).cloned().unwrap_or_default();
                request.headers.extend(headers);
                let new_id = self.next_request_id;
                self.next_request_id += 1;
                self.request_list.insert(new_id, request);
                Resource::new_own(new_id)
            }

            fn body(
                &mut self,
                self_: Resource<$v::standout::app::http::RequestBuilder>,
                body: String,
            ) -> Resource<$v::standout::app::http::RequestBuilder> {
                let id = self_.rep();
                let mut request = self.request_list.get(&id).cloned().unwrap_or_default();
                request.body = body;
                let new_id = self.next_request_id;
                self.next_request_id += 1;
                self.request_list.insert(new_id, request);
                Resource::new_own(new_id)
            }

            fn send(
                &mut self,
                self_: Resource<$v::standout::app::http::RequestBuilder>,
            ) -> Result<$v::standout::app::http::Response, $v::standout::app::http::RequestError> {
                let id = self_.rep();
                match self.request_list.get(&id).cloned() {
                    Some(request) => send_request(&self.client, &request)
                        .map(Into::into)
                        .map_err(Into::into),
                    None => Err($v::standout::app::http::RequestError::Other(
                        "Request not found".to_string(),
                    )),
                }
            }

            fn drop(
                &mut self,
                rep: Resource<$v::standout::app::http::RequestBuilder>,
            ) -> wasmtime::Result<()> {
                self.request_list.remove(&rep.rep());
                Ok(())
            }

            fn object(
                &mut self,
                self_: Resource<$v::standout::app::http::RequestBuilder>,
            ) -> $v::standout::app::http::Request {
                self.request_list
                    .get(&self_.rep())
                    .cloned()
                    .unwrap_or_default()
                    .into()
            }
        }
    };
}

// ============================================================================
// Macro to implement HTTP type conversions for a version
// ============================================================================

macro_rules! impl_http_type_conversions {
    ($v:ident) => {
        impl From<$v::standout::app::http::Method> for Method {
            fn from(m: $v::standout::app::http::Method) -> Self {
                use $v::standout::app::http::Method as V;
                match m {
                    V::Get => Self::Get,
                    V::Post => Self::Post,
                    V::Put => Self::Put,
                    V::Delete => Self::Delete,
                    V::Patch => Self::Patch,
                    V::Options => Self::Options,
                    V::Head => Self::Head,
                }
            }
        }

        impl From<Method> for $v::standout::app::http::Method {
            fn from(m: Method) -> Self {
                match m {
                    Method::Get => Self::Get,
                    Method::Post => Self::Post,
                    Method::Put => Self::Put,
                    Method::Delete => Self::Delete,
                    Method::Patch => Self::Patch,
                    Method::Options => Self::Options,
                    Method::Head => Self::Head,
                }
            }
        }

        impl From<Response> for $v::standout::app::http::Response {
            fn from(r: Response) -> Self {
                Self {
                    status: r.status,
                    headers: r.headers,
                    body: r.body,
                }
            }
        }

        impl From<Request> for $v::standout::app::http::Request {
            fn from(r: Request) -> Self {
                Self {
                    method: r.method.into(),
                    url: r.url,
                    headers: r.headers,
                    body: r.body,
                }
            }
        }

        impl From<RequestError> for $v::standout::app::http::RequestError {
            fn from(e: RequestError) -> Self {
                match e {
                    RequestError::Other(msg) => Self::Other(msg),
                }
            }
        }
    };
}

// ============================================================================
// Generate implementations for all supported versions
// When adding v5, just add:
//   impl_host_request_builder!(v5);
//   impl_http_type_conversions!(v5);
// ============================================================================

impl_host_request_builder!(v3);
impl_host_request_builder!(v4);

impl_http_type_conversions!(v3);
// Note: v4 doesn't need conversions since we use v4 types as the canonical internal types

// ============================================================================
// Shared request sending logic
// ============================================================================

fn send_request(
    client: &std::sync::Arc<std::sync::Mutex<reqwest::blocking::Client>>,
    request: &Request,
) -> Result<Response, RequestError> {
    let client = client.lock().unwrap();
    let mut builder = client.request(request.method.clone().into(), &request.url);

    for (key, value) in &request.headers {
        builder = builder.header(key, value);
    }
    builder = builder.body(request.body.clone());

    match builder.send() {
        Ok(resp) => {
            let headers = resp
                .headers()
                .iter()
                .map(|(k, v)| {
                    (
                        k.as_str().to_string(),
                        v.to_str().unwrap_or_default().to_string(),
                    )
                })
                .collect();

            Ok(Response {
                status: resp.status().as_u16(),
                headers,
                body: resp.text().unwrap_or_default(),
            })
        }
        Err(error) => Err(RequestError::Other(format!(
            "Request failed to {} {}: {}",
            request.method, request.url, error
        ))),
    }
}

// ============================================================================
// Standard type implementations (used by all versions)
// ============================================================================

impl From<Method> for ReqwestMethod {
    fn from(method: Method) -> Self {
        match method {
            Method::Get => Self::GET,
            Method::Post => Self::POST,
            Method::Put => Self::PUT,
            Method::Delete => Self::DELETE,
            Method::Patch => Self::PATCH,
            Method::Head => Self::HEAD,
            Method::Options => Self::OPTIONS,
        }
    }
}

impl Default for Request {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: Method::Get,
            body: String::new(),
            headers: vec![(
                "User-Agent".to_string(),
                format!("Standout-AppBridge/{}", env!("CARGO_PKG_VERSION")),
            )],
        }
    }
}

impl Default for Response {
    fn default() -> Self {
        Self {
            status: 0,
            headers: Vec::new(),
            body: String::new(),
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::{Method::GET, MockServer};

    #[test]
    fn sends_request_with_default_user_agent() {
        use v4::standout::app::http::HostRequestBuilder;

        let version = env!("CARGO_PKG_VERSION");
        let user_agent = format!("Standout-AppBridge/{version}");

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/headers")
                .header("User-Agent", user_agent.clone());
            then.status(200);
        });
        let url = format!("{}/headers", server.base_url());

        let mut app_state = AppState::default();
        let builder = app_state.new();
        let builder = app_state.method(builder, Method::Get);
        let builder = app_state.url(builder, url);

        let response = app_state.send(builder).expect("Request failed");

        assert_eq!(response.status, 200);
        mock.assert();
    }
}
