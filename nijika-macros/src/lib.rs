use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, Lit, parse_macro_input};

#[proc_macro_attribute]
pub fn price(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let ItemFn {
        vis,
        sig,
        block,
        attrs,
    } = input;

    let cost_str = if let Ok(lit) = syn::parse::<Lit>(attr) {
        match lit {
            Lit::Str(s) => s.value(),
            Lit::Float(f) => f.base10_digits().to_string(),
            _ => "0".to_string(),
        }
    } else {
        "0".to_string()
    };

    let fn_name = sig.ident.to_string();
    let service_name = match fn_name.as_str() {
        "remove_bg" => "Background Removal",
        "upscale" => "Image Upscaling",
        _ => &fn_name,
    };

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            use rust_decimal::Decimal;
            use std::str::FromStr;
            use axum::response::IntoResponse;

            let cost = Decimal::from_str(#cost_str).unwrap_or_default();

            let mut tx = match state.db.begin().await {
                Ok(t) => t,
                Err(e) => {
                    tracing::error!("Failed to start transaction: {}", e);
                    return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
                }
            };

            let result = sqlx::query("UPDATE users SET credits = credits - $1 WHERE id = $2 AND credits >= $1")
                .bind(cost)
                .bind(user.id)
                .execute(&mut *tx)
                .await;

            match result {
                Ok(res) if res.rows_affected() == 0 => {
                    let _ = tx.rollback().await;
                    return (axum::http::StatusCode::PAYMENT_REQUIRED, "Insufficient credits").into_response();
                }
                Ok(_) => {},
                Err(e) => {
                    tracing::error!("Failed to deduct credits: {}", e);
                    return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
                }
            }

            if let Err(e) = sqlx::query(
                "INSERT INTO transactions (user_id, amount, type, description) VALUES ($1, $2, $3, $4)"
            )
            .bind(user.id)
            .bind(-cost)
            .bind(crate::models::TransactionType::Charge)
            .bind(format!("Service usage: {}", #service_name))
            .execute(&mut *tx)
            .await {
                tracing::error!("Failed to log transaction: {}", e);
                return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
            }

            let log_id: uuid::Uuid = match sqlx::query_scalar(
                "INSERT INTO usage_logs (user_id, service, status, credits_used) VALUES ($1, $2, $3, $4) RETURNING id"
            )
            .bind(user.id)
            .bind(#service_name)
            .bind("pending")
            .bind(cost)
            .fetch_one(&mut *tx)
            .await {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!("Failed to log pending usage: {}", e);
                    return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
                }
            };

            if let Err(e) = tx.commit().await {
                tracing::error!("Failed to commit upfront deduction: {}", e);
                return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
            }

            let response = (async #block).await.into_response();

            let db = state.db.clone();
            let user_id = user.id;

            if !response.status().is_success() {
                let mut refund_tx = match db.begin().await {
                    Ok(t) => t,
                    Err(_) => return response,
                };

                let _ = sqlx::query("UPDATE users SET credits = credits + $1 WHERE id = $2")
                    .bind(cost)
                    .bind(user_id)
                    .execute(&mut *refund_tx)
                    .await;

                let _ = sqlx::query(
                    "INSERT INTO transactions (user_id, amount, type, description) VALUES ($1, $2, $3, $4)"
                )
                .bind(user_id)
                .bind(cost)
                .bind(crate::models::TransactionType::Refund)
                .bind(format!("Refund for failed service: {}", #service_name))
                .execute(&mut *refund_tx)
                .await;

                let _ = sqlx::query(
                    "UPDATE usage_logs SET status = 'failed' WHERE id = $1"
                )
                .bind(log_id)
                .execute(&mut *refund_tx)
                .await;

                let _ = refund_tx.commit().await;
            } else {
                let _ = sqlx::query(
                    "UPDATE usage_logs SET status = 'success' WHERE id = $1"
                )
                .bind(log_id)
                .execute(&db)
                .await;
            }

            response
        }
    };

    TokenStream::from(expanded)
}
