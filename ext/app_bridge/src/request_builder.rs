use crate::app_state::AppState;
use crate::component::standout::app::http::{
    HostRequestBuilder, Method, Request, RequestBuilder, RequestError, Response,
};
use reqwest::Method as ReqwestMethod;
use std::result::Result::Ok;
use wasmtime::component::Resource;

impl HostRequestBuilder for AppState {
    fn new(&mut self) -> Resource<RequestBuilder> {
        let id = self.next_request_id;
        self.next_request_id += 1;
        self.request_list.insert(id, Request::default());
        Resource::new_own(id)
    }

    fn method(
        &mut self,
        self_: Resource<RequestBuilder>,
        method: Method,
    ) -> Resource<RequestBuilder> {
        let id = self_.rep();
        let mut request = self.request_list.get(&id).cloned().unwrap();
        request.method = method;
        let new_id = self.next_request_id;
        self.next_request_id += 1;
        self.request_list.insert(new_id, request);
        Resource::new_own(new_id)
    }

    fn url(
        &mut self,
        self_: Resource<RequestBuilder>,
        url: wasmtime::component::__internal::String,
    ) -> Resource<RequestBuilder> {
        let id = self_.rep();
        let mut request = self.request_list.get(&id).cloned().unwrap();
        request.url = url;
        let new_id = self.next_request_id;
        self.next_request_id += 1;
        self.request_list.insert(new_id, request);
        Resource::new_own(new_id)
    }

    #[doc = " Add a header to the request"]
    fn header(
        &mut self,
        self_: Resource<RequestBuilder>,
        key: wasmtime::component::__internal::String,
        value: wasmtime::component::__internal::String,
    ) -> Resource<RequestBuilder> {
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
        self_: Resource<RequestBuilder>,
        headers: wasmtime::component::__internal::Vec<(
            wasmtime::component::__internal::String,
            wasmtime::component::__internal::String,
        )>,
    ) -> Resource<RequestBuilder> {
        let id = self_.rep();
        let mut request = self.request_list.get(&id).cloned().unwrap_or_default();
        for (key, value) in headers {
            request.headers.push((key, value));
        }
        let new_id = self.next_request_id;
        self.next_request_id += 1;
        self.request_list.insert(new_id, request);
        Resource::new_own(new_id)
    }

    #[doc = " Add a body to the request"]
    fn body(
        &mut self,
        self_: Resource<RequestBuilder>,
        body: wasmtime::component::__internal::String,
    ) -> Resource<RequestBuilder> {
        let id = self_.rep();
        let mut request = self.request_list.get(&id).cloned().unwrap_or_default();
        request.body = body;
        let new_id = self.next_request_id;
        self.next_request_id += 1;
        self.request_list.insert(new_id, request);
        Resource::new_own(new_id)
    }

    #[doc = " Send the request"]
    fn send(&mut self, self_: Resource<RequestBuilder>) -> Result<Response, RequestError> {
        let id = self_.rep();
        let request = match self.request_list.get(&id).cloned() {
            Some(request) => request,
            None => return Err(RequestError::Other("Request not found".to_string())),
        };
        let client = self.client.lock().unwrap();
        let mut request_builder = client.request(request.method.into(), &request.url);
        for (key, value) in request.headers {
            request_builder = request_builder.header(key, value);
        }
        request_builder = request_builder.body(request.body);

        let response = request_builder.send();

        match response {
            Ok(resp) => {
                let mut response = Response::default();
                response.status = resp.status().as_u16();
                for (key, value) in resp.headers() {
                    response.headers.push((
                        key.as_str().to_string(),
                        value.to_str().unwrap_or_default().to_string(),
                    ));
                }
                response.body = resp.text().unwrap_or_default();
                Ok(response)
            }
            Err(error) => {
                let error_message = format!(
                    "Request failed to {} {}: {}",
                    request.method,
                    request.url,
                    error
                );
                let error = RequestError::Other(error_message);
                Err(error)
            }
        }
    }

    fn drop(&mut self, rep: Resource<RequestBuilder>) -> wasmtime::Result<()> {
        let id = rep.rep();
        self.request_list.remove(&id);
        Ok(())
    }

    fn object(&mut self, self_: Resource<RequestBuilder>) -> Request {
        let id = self_.rep();
        self.request_list.get(&id).cloned().unwrap_or_default()
    }
}

impl From<Method> for ReqwestMethod {
    fn from(method: Method) -> Self {
        match method {
            Method::Get => ReqwestMethod::GET,
            Method::Post => ReqwestMethod::POST,
            Method::Put => ReqwestMethod::PUT,
            Method::Delete => ReqwestMethod::DELETE,
            Method::Patch => ReqwestMethod::PATCH,
            Method::Head => ReqwestMethod::HEAD,
            Method::Options => ReqwestMethod::OPTIONS,
        }
    }
}

impl Default for Request {
    fn default() -> Self {
        let version = env!("CARGO_PKG_VERSION");
        let user_agent = format!("Standout-AppBridge/{version}");
        let headers = vec![
            ("User-Agent".to_string(), user_agent.into()),
        ];

        Self {
            url: "".to_string(),
            method: Method::Get,
            body: "".to_string(),
            headers,
        }
    }
}

impl Default for Response {
    fn default() -> Self {
        Self {
            status: 0,
            headers: Vec::new(),
            body: "".to_string(),
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Delete => write!(f, "DELETE"),
            Method::Patch => write!(f, "PATCH"),
            Method::Head => write!(f, "HEAD"),
            Method::Options => write!(f, "OPTIONS"),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::{MockServer, Method::GET};

    #[test]
    fn sends_request_with_default_user_agent() {
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
