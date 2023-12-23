
use reqwest::header::{HeaderMap, HOST};
use reqwest::{Client, Url, RequestBuilder};
use reqwest_cookie_store::{CookieStoreMutex, RawCookie, CookieStore};
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Deref;
use std::sync::Arc;

/// `Session` is a user-friendly `Client` wrapper, which automatically handles cookies and load/store
/// cookies from/to the specified path.
#[derive(Debug, Clone)]
pub struct Session {
    #[allow(dead_code)] // just make clippy happy
    state: Arc<State>,
    base_url: String,
    client: Client,
}

impl Session {
    /// Try to creates a new `Session` instance, and load cookies from `cookie_store_path`.
    /// When `Session` is dropped(more specifically, when `State` is dropped), it will store cookies
    /// to `cookie_store_path`.
    pub fn new(base_url: String) -> Session {
        let state = State::new();
        let state = Arc::new(state);

        let mut headers = HeaderMap::new();
        headers.append(HOST, base_url.parse().unwrap());

        let client = Client::builder()
            .user_agent("vozForums/366 CFNetwork/1331.0.7 Darwin/21.4.0")
            .default_headers(headers)
            .cookie_provider(state.cookie_store.clone())
            .build().unwrap();

        Session { state, base_url, client }
    }

    pub fn post<U>(&self, path: U) -> RequestBuilder where U: Display {
        self.client.post(format!("https://{0}{path}", self.base_url))
    }

    pub fn get<U>(&self, path: U) -> RequestBuilder where U: Display {
        self.client.get(format!("https://{0}{path}", self.base_url))
    }

    pub fn get_cookies(&self) -> HashMap<String, String> {
        let binding = self.state.cookie_store.lock().unwrap();
        let result = binding.iter_any().map(|x| x.name_value()).map(|(k,v)| (k.to_string(), v.to_string()));
        HashMap::from_iter(result)
    }

    pub fn set_cookie(&self, key: String, value: String) {
        let cookie = RawCookie::build(key, value).finish();
        let url = format!("https://{0}", self.base_url).parse::<Url>().unwrap();
        self.state.cookie_store.lock().unwrap().insert_raw(&cookie, &url).ok();
    }
}

impl Deref for Session {
    type Target = Client;
    fn deref(&self) -> &Client {
        &self.client
    }
}

#[derive(Debug)]
struct State {
    cookie_store: Arc<CookieStoreMutex>,
}

impl State {
    pub fn new() -> State {
        let cookie_store = CookieStore::default();
        let cookie_store = Arc::new(CookieStoreMutex::new(cookie_store));

        State {
            cookie_store,
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        
    }
}