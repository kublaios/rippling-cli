use crate::persistence;
use attohttpc::header::IntoHeaderName;
pub use attohttpc::Method;
pub use attohttpc::StatusCode;
use serde::de::DeserializeOwned;
use url::Url;

use super::Error;

/// Wraps the 3rd party http client RequestBuilder
pub struct Request {
    builder: attohttpc::RequestBuilder,
}

pub type Result<T> = std::result::Result<T, super::Error>;

impl Request {
    pub fn new(method: Method, path: &str) -> Self {
        let builder = attohttpc::RequestBuilder::new(method, Self::url_for(path));
        Self { builder: builder }
    }

    pub fn bearer_auth(self, token: &str) -> Self {
        Self { builder: self.builder.bearer_auth(token) }
    }

    pub fn header<K: IntoHeaderName>(self, key: K, value: String) -> Self {
        Self { builder: self.builder.header(key, value) }
    }

    pub fn param<K: AsRef<str>, V: ToString>(self, key: K, value: V) -> Self {
        Self { builder: self.builder.param(key, value) }
    }

    pub fn send(self) -> Result<Response> {
        Ok(Response::new(self.builder.send()?))
    }

    pub fn send_json<J: serde::Serialize>(self, json: J) -> Result<Response> {
        Ok(Response::new(self.builder.json(&json)?.send()?))
    }

    fn url_for(path: &str) -> Url {
        #[cfg(not(test))]
        let url = "https://app.rippling.com/api/";
        #[cfg(test)]
        let url = &utilities::mocking::server_url();
        Url::parse(url).unwrap().join(path).unwrap()
    }
}

pub struct Response {
    response: attohttpc::Response,
    parse_states: Vec<StatusCode>,
}

impl Response {
    pub fn new(response: attohttpc::Response) -> Self {
        Self { response, parse_states: vec![StatusCode::OK, StatusCode::CREATED] }
    }

    pub fn accept_states(self, states: Vec<StatusCode>) -> Self {
        Self { response: self.response, parse_states: states }
    }

    pub fn into_error(self) -> Error {
        <Error as From<attohttpc::Response>>::from(self.response)
    }

    pub fn parse_json<J>(self) -> Result<J>
    where
        J: DeserializeOwned,
    {
        let res = self.response;
        if self.parse_states.contains(&res.status()) {
            Ok(res.json::<J>()?)
        } else {
            Err(res.into())
        }
    }

    pub fn status(&self) -> StatusCode {
        self.response.status()
    }
}

#[derive(Clone, Debug)]
pub struct Session {
    access_token: String,
    pub company: Option<String>,
    pub role: Option<String>,
}

impl Session {
    pub fn new(token: String) -> Self {
        Self { access_token: token, company: None, role: None }
    }

    #[allow(dead_code)]
    pub fn load() -> Self {
        let state = persistence::State::load();
        Self {
            access_token: state.access_token.expect("State missing access token"),
            company: state.company_id,
            role: state.role_id,
        }
    }

    pub fn save(&self) {
        let state = persistence::State {
            access_token: Some(self.access_token.clone()),
            company_id: self.company.clone(),
            role_id: self.role.clone(),
        };
        state.store();
    }

    pub fn set_company_and_role(&mut self, company: String, role: String) {
        self.company = Some(company);
        self.role = Some(role);
    }

    pub fn company(&self) -> Option<&str> {
        self.company.as_ref().map(|s| s.as_str())
    }

    pub fn role(&self) -> Option<&str> {
        self.role.as_ref().map(|s| s.as_str())
    }

    pub fn get(&self, path: &str) -> Request {
        self.request(Method::GET, path)
    }

    pub fn get_json<J: DeserializeOwned>(&self, path: &str) -> Result<J> {
        self.request(Method::GET, path).send()?.parse_json::<J>()
    }

    pub fn post(&self, path: &str) -> Request {
        self.request(Method::POST, path)
    }

    fn request(&self, method: Method, path: &str) -> Request {
        let mut builder = Request::new(method, path).bearer_auth(&self.access_token);
        if let Some(value) = &self.company {
            builder = builder.header("company", value.to_owned());
        }
        if let Some(value) = &self.role {
            builder = builder.header("role", value.to_owned());
        }
        builder
    }
}
