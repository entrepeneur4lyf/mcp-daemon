use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures::future::ready;
use futures::future::{LocalBoxFuture, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

/// Claims in a JWT token
///
/// This struct represents the standard claims in a JWT token,
/// including expiration and issued-at timestamps.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Expiration timestamp
    pub exp: usize,
    /// Issued at timestamp
    pub iat: usize,
}

/// Configuration for JWT authentication
///
/// This struct contains the configuration for JWT authentication,
/// including the secret key used for token verification.
#[derive(Clone)]
pub struct AuthConfig {
    /// Secret key for JWT signing and verification
    pub jwt_secret: String,
}

/// JWT authentication handler
///
/// This struct implements the Transform trait for JWT authentication
/// in Actix Web applications. It verifies JWT tokens in the Authorization
/// header and rejects requests with invalid or missing tokens.
///
/// # Examples
///
/// ```
/// use actix_web::{App, web, HttpServer};
/// use mcp_daemon::sse::middleware::{JwtAuth, AuthConfig};
///
/// let jwt_secret = "your-secret-key".to_string();
/// let auth_config = Some(AuthConfig { jwt_secret });
///
/// let app = App::new()
///     .wrap(JwtAuth::new(auth_config))
///     .route("/protected", web::get().to(|| async { "Protected resource" }));
/// ```
pub struct JwtAuth(Option<AuthConfig>);

impl JwtAuth {
    /// Creates a new JWT authentication handler
    ///
    /// # Arguments
    ///
    /// * `config` - Optional authentication configuration. If None, authentication is disabled.
    ///
    /// # Returns
    ///
    /// A new JwtAuth instance
    pub fn new(config: Option<AuthConfig>) -> Self {
        JwtAuth(config)
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service,
            auth_config: self.0.clone(),
        }))
    }
}

/// Middleware for handling JWT authentication
///
/// This middleware verifies JWT tokens in incoming requests and
/// rejects requests with invalid or missing tokens when authentication
/// is enabled.
pub struct JwtAuthMiddleware<S> {
    service: S,
    auth_config: Option<AuthConfig>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if let Some(config) = &self.auth_config {
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok());

            match auth_header {
                Some(auth) if auth.starts_with("Bearer ") => {
                    let token = &auth[7..];
                    match decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
                        &Validation::default(),
                    ) {
                        Ok(_) => {
                            let fut = self.service.call(req);
                            Box::pin(
                                async move { fut.await.map(ServiceResponse::map_into_left_body) },
                            )
                        }
                        Err(_) => {
                            let (req, _) = req.into_parts();
                            Box::pin(async move {
                                Ok(
                                    ServiceResponse::new(
                                        req,
                                        HttpResponse::Unauthorized().finish(),
                                    )
                                    .map_into_right_body(),
                                )
                            })
                        }
                    }
                }
                _ => {
                    let (req, _) = req.into_parts();
                    Box::pin(async move {
                        Ok(
                            ServiceResponse::new(req, HttpResponse::Unauthorized().finish())
                                .map_into_right_body(),
                        )
                    })
                }
            }
        } else {
            let fut = self.service.call(req);
            Box::pin(async move { fut.await.map(ServiceResponse::map_into_left_body) })
        }
    }
}
