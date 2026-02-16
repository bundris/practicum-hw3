use blog_client::{AuthResponse, Post};
use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::prelude::*;

const DEFAULT_BASE_URL: &str = "http://localhost:3000";
const TOKEN_KEY: &str = "blog_token";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PostsListResponse {
    posts: Vec<Post>,
    total: i64,
    limit: i32,
    offset: i32,
}

#[wasm_bindgen]
pub struct BlogApp {
    base_url: String,
    token: Option<String>,
}

#[wasm_bindgen]
impl BlogApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> BlogApp {
        let token = get_token_from_storage().ok().flatten();
        BlogApp {
            base_url: DEFAULT_BASE_URL.to_string(),
            token,
        }
    }

    #[wasm_bindgen(js_name = loadToken)]
    pub fn load_token(&mut self) -> Result<JsValue, JsValue> {
        let token = get_token_from_storage()?;
        self.token = token.clone();
        to_js_value(&token)
    }

    #[wasm_bindgen(js_name = saveToken)]
    pub fn save_token(&mut self, token: String) -> Result<JsValue, JsValue> {
        save_token_to_storage(&token)?;
        self.token = Some(token);
        Ok(JsValue::TRUE)
    }

    #[wasm_bindgen(js_name = clearToken)]
    pub fn clear_token(&mut self) -> Result<JsValue, JsValue> {
        clear_token_from_storage()?;
        self.token = None;
        Ok(JsValue::TRUE)
    }

    #[wasm_bindgen]
    pub fn is_authenticated(&self) -> Result<JsValue, JsValue> {
        Ok(JsValue::from_bool(self.token.is_some()))
    }

    #[wasm_bindgen]
    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<JsValue, JsValue> {
        validate_non_empty("username", &username)?;
        validate_non_empty("email", &email)?;
        validate_non_empty("password", &password)?;

        let url = format!("{}/api/auth/register", self.base_url);
        let payload = json!({
            "username": username,
            "email": email,
            "password": password,
        });

        let response = gloo_net::http::Request::post(&url)
          .json(&payload)
          .map_err(to_js_error)?
          .send()
          .await
          .map_err(to_js_error)?;

        if !response.ok() {
            let text = response.text().await.unwrap_or_else(|_| "Registration failed".into());
            return Err(JsValue::from_str(&text));
        }

        let auth: AuthResponse = response.json().await.map_err(to_js_error)?;
        save_token_to_storage(&auth.token)?;
        self.token = Some(auth.token.clone());
        to_js_value(&auth)
    }

    #[wasm_bindgen]
    pub async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<JsValue, JsValue> {
        validate_non_empty("username", &username)?;
        validate_non_empty("password", &password)?;

        let url = format!("{}/api/auth/login", self.base_url);
        let payload = json!({
            "username": username,
            "password": password,
        });

        let response = gloo_net::http::Request::post(&url)
          .json(&payload)
          .map_err(to_js_error)?
          .send()
          .await
          .map_err(to_js_error)?;

        if !response.ok() {
            let text = response.text().await.unwrap_or_else(|_| "Login failed".into());
            return Err(JsValue::from_str(&text));
        }

        let auth: AuthResponse = response.json().await.map_err(to_js_error)?;
        save_token_to_storage(&auth.token)?;
        self.token = Some(auth.token.clone());
        to_js_value(&auth)
    }

    #[wasm_bindgen(js_name = load_posts)]
    pub async fn load_posts(&self) -> Result<JsValue, JsValue> {
        let url = format!("{}/api/posts?limit=50&offset=0", self.base_url);

        let response = gloo_net::http::Request::get(&url)
            .send()
            .await
            .map_err(to_js_error)?;

        if !response.ok() {
            let text = response.text().await.unwrap_or_else(|_| "Failed to load posts".into());
            return Err(JsValue::from_str(&text));
        }

        let list: PostsListResponse = response.json().await.map_err(to_js_error)?;
        to_js_value(&list.posts)
    }

    #[wasm_bindgen]
    pub async fn create_post(&self, title: String, content: String) -> Result<JsValue, JsValue> {
        validate_non_empty("title", &title)?;
        validate_non_empty("content", &content)?;
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Unauthorized"))?;

        let url = format!("{}/api/posts", self.base_url);
        let payload = json!({
            "title": title,
            "content": content,
        });

        let response = gloo_net::http::Request::post(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .json(&payload)
            .map_err(to_js_error)?
            .send()
            .await
            .map_err(to_js_error)?;

        if !response.ok() {
            let text = response.text().await.unwrap_or_else(|_| "Create failed".into());
            return Err(JsValue::from_str(&text));
        }

        let post: Post = response.json().await.map_err(to_js_error)?;
        to_js_value(&post)
    }

    #[wasm_bindgen]
    pub async fn update_post(
        &self,
        id: i64,
        title: String,
        content: String,
    ) -> Result<JsValue, JsValue> {
        validate_non_empty("title", &title)?;
        validate_non_empty("content", &content)?;
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Unauthorized"))?;

        let url = format!("{}/api/posts/{}", self.base_url, id);
        let payload = json!({
            "title": title,
            "content": content,
        });

        let response = gloo_net::http::Request::put(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .json(&payload)
            .map_err(to_js_error)?
            .send()
            .await
            .map_err(to_js_error)?;


        if !response.ok() {
            let text = response.text().await.unwrap_or_else(|_| "Update failed".into());
            return Err(JsValue::from_str(&text));
        }

        let post: Post = response.json().await.map_err(to_js_error)?;
        to_js_value(&post)
    }

    #[wasm_bindgen]
    pub async fn delete_post(&self, id: i64) -> Result<JsValue, JsValue> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Unauthorized"))?;

        let url = format!("{}/api/posts/{}", self.base_url, id);

        let response = gloo_net::http::Request::delete(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(to_js_error)?;

        if !response.ok() {
            let text = response.text().await.unwrap_or_else(|_| "Delete failed".into());
            return Err(JsValue::from_str(&text));
        }

        Ok(JsValue::TRUE)
    }
}

fn validate_non_empty(field: &str, value: &str) -> Result<(), JsValue> {
    if value.trim().is_empty() {
        Err(JsValue::from_str(&format!("{} is required", field)))
    } else {
        Ok(())
    }
}

fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|err| JsValue::from_str(&err.to_string()))
}

fn to_js_error<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}

fn to_js_error_value(err: JsValue) -> JsValue {
    err
}

fn save_token_to_storage(token: &str) -> Result<(), JsValue> {
    let storage = local_storage()?;
    storage
        .set_item(TOKEN_KEY, token)
        .map_err(to_js_error_value)?;
    Ok(())
}

fn clear_token_from_storage() -> Result<(), JsValue> {
    let storage = local_storage()?;
    storage
        .remove_item(TOKEN_KEY)
        .map_err(to_js_error_value)?;
    Ok(())
}

fn get_token_from_storage() -> Result<Option<String>, JsValue> {
    let storage = local_storage()?;
    storage.get_item(TOKEN_KEY).map_err(to_js_error_value)
}

fn local_storage() -> Result<web_sys::Storage, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    window
        .local_storage()
        .map_err(to_js_error_value)?
        .ok_or_else(|| JsValue::from_str("LocalStorage unavailable"))
}
