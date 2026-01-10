use salvo::Router;

pub async fn auth_router() -> Router {
    Router::with_path("auth")
}
