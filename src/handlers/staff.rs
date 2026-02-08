use crate::AppState;
use crate::models::{PaginationMetadata, Transaction, TransactionType, UsageLog, User, UserRole};
use askama::Template;
use axum::{
    Extension, Form,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "staff/users.html")]
struct UsersTemplate {
    #[allow(dead_code)]
    user: User,
    users: Vec<User>,
    pagination: PaginationMetadata,
}

#[derive(Template)]
#[template(path = "staff/user_detail.html")]
struct UserDetailTemplate {
    user: User,
    target_user: User,
    logs: Vec<UsageLog>,
    transactions: Vec<Transaction>,
}

#[derive(Template)]
#[template(path = "staff/logs.html")]
struct LogsTemplate {
    #[allow(dead_code)]
    user: User,
    logs: Vec<UsageLog>,
    pagination: PaginationMetadata,
}

#[derive(Template)]
#[template(path = "staff/system.html")]
struct SystemTemplate {
    #[allow(dead_code)]
    user: User,
    total_users: i64,
    total_credits: Decimal,
    total_images_processed: i64,
    success_rate: f64,
}

#[derive(Deserialize)]
pub struct StaffQuery {
    page: Option<i64>,
}

pub async fn users_list(
    State(state): State<AppState>,
    Query(query): Query<StaffQuery>,
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, Response> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = 20;
    let offset = (page - 1) * page_size;

    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    let users = sqlx::query_as::<_, User>(
        "SELECT id, github_id, gitlab_id, email, username, avatar_url, credits, role, is_active, api_key, oauth_account_created_at, created_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    let pagination = PaginationMetadata {
        current_page: page,
        total_pages: (total_users + page_size - 1) / page_size,
        total_items: total_users,
        page_size,
    };

    Ok(Html(
        UsersTemplate {
            user,
            users,
            pagination,
        }
        .render()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?,
    ))
}

pub async fn user_detail(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, Response> {
    let target_user = sqlx::query_as::<_, User>(
        "SELECT id, github_id, gitlab_id, email, username, avatar_url, credits, role, is_active, api_key, oauth_account_created_at, created_at FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
    .ok_or_else(|| StatusCode::NOT_FOUND.into_response())?;

    let logs = sqlx::query_as::<_, UsageLog>(
        "SELECT id, user_id, service, status, details, credits_used, created_at FROM usage_logs WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50"
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    let transactions = sqlx::query_as::<_, Transaction>(
        "SELECT id, user_id, amount, type, description, created_at FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50"
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    Ok(Html(
        UserDetailTemplate {
            user,
            target_user,
            logs,
            transactions,
        }
        .render()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?,
    ))
}

pub async fn audit_logs(
    State(state): State<AppState>,
    Query(query): Query<StaffQuery>,
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, Response> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = 50;
    let offset = (page - 1) * page_size;

    let total_logs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM usage_logs")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    let logs = sqlx::query_as::<_, UsageLog>(
        "SELECT id, user_id, service, status, details, credits_used, created_at FROM usage_logs ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    let pagination = PaginationMetadata {
        current_page: page,
        total_pages: (total_logs + page_size - 1) / page_size,
        total_items: total_logs,
        page_size,
    };

    Ok(Html(
        LogsTemplate {
            user,
            logs,
            pagination,
        }
        .render()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?,
    ))
}

#[derive(Deserialize)]
pub struct AdjustCreditsForm {
    amount: Decimal,
    reason: String,
}

pub async fn adjust_credits(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Form(form): Form<AdjustCreditsForm>,
) -> Result<impl IntoResponse, Response> {
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    let tx_type = if form.amount >= Decimal::ZERO {
        TransactionType::Bonus
    } else {
        TransactionType::Charge
    };

    sqlx::query("UPDATE users SET credits = credits + $1 WHERE id = $2")
        .bind(form.amount)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    sqlx::query(
        "INSERT INTO transactions (user_id, amount, type, description) VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(form.amount)
    .bind(tx_type)
    .bind(form.reason)
    .execute(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    Ok(axum::response::Redirect::to(&format!(
        "/staff/users/{}",
        user_id
    )))
}

pub async fn toggle_user_status(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, Response> {
    sqlx::query("UPDATE users SET is_active = NOT is_active WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    Ok(axum::response::Redirect::to(&format!(
        "/staff/users/{}",
        user_id
    )))
}

#[derive(Deserialize)]
pub struct ChangeRoleForm {
    role: UserRole,
}

pub async fn change_role(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Form(form): Form<ChangeRoleForm>,
) -> Result<impl IntoResponse, Response> {
    sqlx::query("UPDATE users SET role = $1 WHERE id = $2")
        .bind(form.role)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    Ok(axum::response::Redirect::to(&format!(
        "/staff/users/{}",
        user_id
    )))
}

pub async fn system_status(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, Response> {
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    let total_credits: Decimal =
        sqlx::query_scalar::<_, Option<Decimal>>("SELECT SUM(credits) FROM users")
            .fetch_one(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
            .unwrap_or(Decimal::ZERO);

    let total_images_processed: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM usage_logs WHERE status = 'success'")
            .fetch_one(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    let total_attempts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM usage_logs")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    let success_rate = if total_attempts > 0 {
        (total_images_processed as f64 / total_attempts as f64) * 100.0
    } else {
        100.0
    };

    Ok(Html(
        SystemTemplate {
            user,
            total_users,
            total_credits,
            total_images_processed,
            success_rate,
        }
        .render()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?,
    ))
}
