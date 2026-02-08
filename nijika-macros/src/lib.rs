use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, Lit, parse_macro_input};

#[proc_macro_attribute]
pub fn price(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Expect a string literal for price to avoid precision issues with float in tokens
    let cost_str = if let Ok(lit) = syn::parse::<Lit>(attr) {
        match lit {
            Lit::Str(s) => s.value(),
            Lit::Float(f) => f.base10_digits().to_string(),
            _ => "0".to_string(),
        }
    } else {
        "0".to_string()
    };

    let input = parse_macro_input!(item as ItemFn);
    let ItemFn {
        vis,
        sig,
        block,
        attrs,
    } = input;

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            use rust_decimal::Decimal;
            use std::str::FromStr;
            use axum::response::IntoResponse;

            let cost = Decimal::from_str(#cost_str).unwrap_or_default();

            // Check credits first
            if user.credits < cost {
                return (axum::http::StatusCode::PAYMENT_REQUIRED, "Insufficient credits").into_response();
            }

            // Execute the handler
            let response = (async move #block).await.into_response();

            // Deduct credits if successful
            if response.status().is_success() {
                let _ = sqlx::query("UPDATE users SET credits = credits - $1 WHERE id = $2")
                    .bind(cost)
                    .bind(user.id)
                    .execute(&state.db)
                    .await;
            }

            response
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn ratelimit(attr: TokenStream, item: TokenStream) -> TokenStream {
    let rps = if let Ok(lit) = syn::parse::<syn::LitInt>(attr) {
        lit.base10_parse::<u32>().unwrap_or(5)
    } else {
        5
    };

    let input = parse_macro_input!(item as ItemFn);
    let ItemFn {
        vis,
        sig,
        block,
        attrs,
    } = input;

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            use std::net::SocketAddr;
            use axum::response::IntoResponse;
            use governor::{Quota, RateLimiter};
            use once_cell::sync::Lazy;
            use dashmap::DashMap;
            use std::num::NonZeroU32;
            use std::sync::Arc;

            static LIMITERS: Lazy<DashMap<SocketAddr, Arc<RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>>> = Lazy::new(DashMap::new);

            let addr = request.extensions().get::<axum::extract::ConnectInfo<SocketAddr>>()
                .map(|ci| ci.0)
                .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 0)));

            let limiter = LIMITERS.entry(addr).or_insert_with(|| {
                Arc::new(RateLimiter::direct(Quota::per_second(NonZeroU32::new(#rps).unwrap())))
            });

            if let Err(_) = limiter.check() {
                return (axum::http::StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response();
            }

            (async move #block).await.into_response()
        }
    };

    TokenStream::from(expanded)
}
