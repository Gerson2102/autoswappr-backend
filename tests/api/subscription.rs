use axum::{
    body::Body,
    http::{header::CONTENT_TYPE, Request, StatusCode},
};
use serde_json::json;

use crate::helpers::*;

#[tokio::test]
async fn test_subscribe_ok() {
    let app = TestApp::new().await;

    clean_database(&app.db.pool).await;

    let payload = json!({
        "wallet_address": "0xdbfcab49bd9bced4636b04319d71fbd0d84bde78a1d38e9e2fc391e83187c1c3",
        "to_token": "0xde3bc70e81af42a996a559a60f0fdf1cb371f012790f1b30de709efa637b9af5",
        "from_token": [
            "0x07ab8059db97aab8ced83b37a1d60b8eef540f6cdc96acc153d583a59bedd125",
            "0x40ca979f20ed76f960dc719457eaf0cef3b2c3932d58435b9192a58bc56c1e40"
        ],
        "percentage": [60, 40]
    });

    let req = Request::builder()
        .method("POST")
        .uri("/subscriptions")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let resp = app.request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_successful_subscription_creation() {
    let app = TestApp::new().await;

    clean_database(&app.db.pool).await;

    let wallet_address = "0x40ca979f20ed76f960dc719457eaf0cef3b2c3932d58435b9192a58bc56c1e40";
    let to_token = "0xdbfcab49bd9bced4636b04319d71fbd0d84bde78a1d38e9e2fc391e83187c1c3";
    let from_tokens = vec![
        "0x07ab8059db97aab8ced83b37a1d60b8eef540f6cdc96acc153d583a59bedd125",
        "0x40ca979f20ed76f960dc719457eaf0cef3b2c3932d58435b9192a58bc56c1e40",
    ];
    let percentages = vec![60, 40];

    let payload = json!({
        "wallet_address": wallet_address,
        "to_token": to_token,
        "from_token": from_tokens,
        "percentage": percentages
    });

    let req = Request::builder()
        .method("POST")
        .uri("/subscriptions")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let resp = app.request(req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let subscription = sqlx::query!(
        r#"
        SELECT wallet_address, to_token, is_active
        FROM swap_subscription
        WHERE wallet_address = $1
        "#,
        wallet_address
    )
    .fetch_one(&app.db.pool)
    .await
    .unwrap();

    assert_eq!(subscription.wallet_address, wallet_address);
    assert_eq!(subscription.to_token, to_token);
    assert!(subscription.is_active);

    let from_token_records = sqlx::query!(
        r#"
        SELECT from_token, percentage
        FROM swap_subscription_from_token
        WHERE wallet_address = $1
        "#,
        wallet_address
    )
    .fetch_all(&app.db.pool)
    .await
    .unwrap();

    assert_eq!(from_token_records.len(), 2);

    let token_percentages: std::collections::HashMap<&str, i16> = from_token_records
        .iter()
        .map(|record| (record.from_token.as_str(), record.percentage))
        .collect();

    assert_eq!(
        token_percentages.get(from_tokens[0]),
        Some(&(percentages[0] as i16)),
        "First token {} should have percentage {}",
        from_tokens[0],
        percentages[0]
    );

    assert_eq!(
        token_percentages.get(from_tokens[1]),
        Some(&(percentages[1] as i16)),
        "Second token {} should have percentage {}",
        from_tokens[1],
        percentages[1]
    );
}

#[tokio::test]
async fn test_invalid_percentage_length() {
    let app = TestApp::new().await;

    clean_database(&app.db.pool).await;

    let payload = json!({
        "wallet_address": "0x40ca979f20ed76f960dc719457eaf0cef3b2c3932d58435b9192a58bc56c1e40",
        "to_token": "0x07ab8059db97aab8ced83b37a1d60b8eef540f6cdc96acc153d583a59bedd125",
        "from_token": [
            "0xdbfcab49bd9bced4636b04319d71fbd0d84bde78a1d38e9e2fc391e83187c1c3",
            "0xde3bc70e81af42a996a559a60f0fdf1cb371f012790f1b30de709efa637b9af5"
        ],
        "percentage": [20]
    });

    let req = Request::builder()
        .method("POST")
        .uri("/subscriptions")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let resp = app.request(req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invalid_wallet_address() {
    let app = TestApp::new().await;

    clean_database(&app.db.pool).await;

    let payload = json!({
        "wallet_address": "invalid_wallet_address",
        "to_token": "0xdbfcab49bd9bced4636b04319d71fbd0d84bde78a1d38e9e2fc391e83187c1c3",
        "from_token": [
            "0x07ab8059db97aab8ced83b37a1d60b8eef540f6cdc96acc153d583a59bedd125",
            "0x40ca979f20ed76f960dc719457eaf0cef3b2c3932d58435b9192a58bc56c1e40"
        ],
        "percentage": [20, 80]
    });

    let req = Request::builder()
        .method("POST")
        .uri("/subscriptions")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let resp = app.request(req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invalid_to_token_address() {
    let app = TestApp::new().await;

    clean_database(&app.db.pool).await;

    let payload = json!({
        "wallet_address": "0x07ab8059db97aab8ced83b37a1d60b8eef540f6cdc96acc153d583a59bedd125",
        "to_token": "invalid_to_token",
        "from_token": [
            "0x40ca979f20ed76f960dc719457eaf0cef3b2c3932d58435b9192a58bc56c1e40",
            "0xdbfcab49bd9bced4636b04319d71fbd0d84bde78a1d38e9e2fc391e83187c1c3"
        ],
        "percentage": [20, 80]
    });

    let req = Request::builder()
        .method("POST")
        .uri("/subscriptions")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let resp = app.request(req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_successful_subscription_retrieval() {
    let app = TestApp::new().await;

    let req = Request::builder()
        .method("GET")
        .uri("/subscriptions?wallet_address=0xdbfcab49bd9bced4636b04319d71fbd0d84bde78a1d38e9e2fc391e83187c1c3")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::empty())
        .unwrap();
    let resp = app.request(req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
