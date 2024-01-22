
use nom::AsBytes;
use reqwest::header::{HeaderMap, HOST};
use reqwest::{Client, Url, Request, Response};
use reqwest_cookie_store::{CookieStoreMutex, RawCookie, CookieStore};
use reqwest_middleware::{Middleware, Next, ClientBuilder, ClientWithMiddleware, RequestBuilder};
use select::document::Document;
use select::predicate::Name;
use task_local_extensions::Extensions;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

/// `Session` is a user-friendly `Client` wrapper, which automatically handles cookies and load/store
/// cookies from/to the specified path.
#[derive(Debug, Clone)]
pub struct Session {
    #[allow(dead_code)] // just make clippy happy
    state: Arc<State>,
    base_url: String,
    client: ClientWithMiddleware,
}

#[async_trait::async_trait]
impl Middleware for State {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        let mut res = next.run(req, extensions).await?;
        let content = res.chunk().await?.ok_or(reqwest_middleware::Error::Middleware(anyhow::anyhow!("")))?;
        let document = Document::from_read(content.as_bytes()).ok().ok_or(reqwest_middleware::Error::Middleware(anyhow::anyhow!("")))?;
        let csrf = document.find(Name("html")).next().and_then(|n| n.attr("data-csrf")).map(str::to_string);
        *self.csrf.lock().unwrap() = csrf;
        reqwest_middleware::Result::Ok(res)
    }
}

impl Session {
    /// Try to creates a new `Session` instance, and load cookies from `cookie_store_path`.
    /// When `Session` is dropped(more specifically, when `State` is dropped), it will store cookies
    /// to `cookie_store_path`.
    pub fn new(base_url: String) -> Session {
        let state_raw = State::new();
        let state = Arc::new(state_raw);

        let mut headers = HeaderMap::new();
        headers.append(HOST, base_url.parse().unwrap());

        let client_raw = Client::builder()
            .user_agent("vozForums/366 CFNetwork/1331.0.7 Darwin/21.4.0")
            .default_headers(headers)
            .cookie_provider(state.cookie_store.clone())
            .build().unwrap();

        let client = ClientBuilder::new(client_raw).with_arc(state.clone()).build();

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

    pub fn get_csrf(&self) -> Option<String> {
        let result = self.state.csrf.lock().unwrap();
        result.clone()
    }
}

impl Deref for Session {
    type Target = ClientWithMiddleware;
    fn deref(&self) -> &ClientWithMiddleware {
        &self.client
    }
}

#[derive(Debug)]
struct State {
    cookie_store: Arc<CookieStoreMutex>,
    csrf: Arc<Mutex<Option<String>>>
}

impl State {
    pub fn new() -> State {
        let cookie_store = CookieStore::default();
        let cookie_store = Arc::new(CookieStoreMutex::new(cookie_store));
        let csrf = Arc::new(Mutex::<Option<String>>::new(None));
        State {
            cookie_store,
            csrf: csrf
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        
    }
}