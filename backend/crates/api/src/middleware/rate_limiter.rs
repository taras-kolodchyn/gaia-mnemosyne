use axum::http::Request;
use axum::{body::Body, http::StatusCode, middleware::Next, response::Response};
use redis::AsyncCommands;

const MAX_PER_MINUTE: i64 = 20;

fn extract_session_id<B>(req: &Request<B>) -> String {
    if let Some(q) = req.uri().query() {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                if k == "session" {
                    return v.to_string();
                }
            }
        }
    }
    if let Some(val) = req.headers().get("x-session-id") {
        if let Ok(s) = val.to_str() {
            return s.to_string();
        }
    }
    "anonymous".to_string()
}

pub async fn rate_limit(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let session_id = extract_session_id(&req);
    let redis_url =
        std::env::var("MNEMO_REDIS_URL").unwrap_or_else(|_| "redis://redis:6379".to_string());

    let client = redis::Client::open(redis_url).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut conn =
        client.get_async_connection().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // key per minute bucket
    let now_minute = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 60) as i64;
    let key = format!("rl:session:{}:{}", session_id, now_minute);

    let count: i64 = conn.incr(&key, 1).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if count == 1 {
        let _: Result<(), _> = conn.expire(&key, 60).await;
    }

    if count > MAX_PER_MINUTE {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}
