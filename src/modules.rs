use salvo::{oapi::extract::JsonBody, prelude::*};
use validator::Validate;

use crate::dto::MessageRes;

mod auth;

#[handler]
async fn hello() -> &'static str {
    "Hello World"
}

pub async fn modules_router() -> Router {
    Router::with_path("api")
        .get(hello)
        .post(push_msg)
        .push(auth::auth_router().await)
}

#[handler]
async fn push_msg(req: JsonBody<MessageRes>, res: &mut Response) {
    let body = req.into_inner();
    tracing::info!("Received message: {:?}", body);
    let _ = res.add_header("Content-Type", "application/json", true);
    if let Err(e) = body.validate() {
        res.status_code(StatusCode::BAD_REQUEST).render(Json(e));
        return;
    }
    match crate::configs::kafka::kafka_producer().await {
        Ok(producer) => {
            let key = uuid::Uuid::now_v7().to_string();
            let payload = body.message.unwrap_or_default();
            let key_clone = key.clone();
            let record = rdkafka::producer::FutureRecord::to("test")
                .payload(&payload)
                .key(key_clone.as_str());
            match producer
                .send(record, std::time::Duration::from_secs(0))
                .await
            {
                Ok(delivery) => {
                    tracing::info!("Message delivered: {:?}", delivery);
                }
                Err((e, _)) => {
                    tracing::error!("Failed to deliver message: {:?}", e);
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR)
                        .render(Json(MessageRes {
                            message: Some(format!("Failed to deliver message: {}", e)),
                        }));
                    return;
                }
            }
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR)
                .render(Json(MessageRes {
                    message: Some(format!("Failed to create Kafka producer: {}", e)),
                }));
            return;
        }
    }
    res.render(Json(MessageRes {
        message: Some("Message received".to_string()),
    }));
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
