use salvo::prelude::*;

mod auth;

#[handler]
async fn hello() -> &'static str {
    "Hello World"
}

pub async fn modules_router() -> Router {
    Router::with_path("api")
        .get(hello)
        .push(auth::auth_router().await)
}

#[cfg(test)]
mod tests {
    use salvo::prelude::*;
    use salvo::test::{ResponseExt, TestClient};

    #[tokio::test]
    async fn test_hello_world() {
        let service = Service::new(super::modules_router().await);
        let mut response = TestClient::get(format!("http://127.0.0.1:8698/api"))
            .send(&service)
            .await;
        let opt_status = response.status_code;
        assert!(opt_status.is_some());
        assert_eq!(StatusCode::OK, opt_status.unwrap());
        let content = response.take_string().await.unwrap();
        assert_eq!(content, "Hello World");
    }
}
