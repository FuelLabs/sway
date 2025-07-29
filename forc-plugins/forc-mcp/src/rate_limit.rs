use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_day: u32,
}

/// Request tracking for a specific key (IP or IP+API key)
#[derive(Debug, Clone)]
struct RequestTracker {
    minute_requests: Vec<Instant>,
    day_requests: Vec<Instant>,
    last_cleanup: Instant,
}

impl RequestTracker {
    fn new() -> Self {
        Self {
            minute_requests: Vec::new(),
            day_requests: Vec::new(),
            last_cleanup: Instant::now(),
        }
    }

    /// Clean up expired requests and check if new request is allowed
    fn check_and_add_request(&mut self, config: &RateLimitConfig) -> bool {
        let now = Instant::now();

        // Clean up old requests periodically
        if now.duration_since(self.last_cleanup) > Duration::from_secs(60) {
            self.cleanup_expired_requests(now);
            self.last_cleanup = now;
        }

        // Check minute limit
        let minute_cutoff = now - Duration::from_secs(60);
        self.minute_requests.retain(|&time| time > minute_cutoff);

        if self.minute_requests.len() >= config.requests_per_minute as usize {
            return false;
        }

        // Check day limit
        let day_cutoff = now - Duration::from_secs(24 * 60 * 60);
        self.day_requests.retain(|&time| time > day_cutoff);

        if self.day_requests.len() >= config.requests_per_day as usize {
            return false;
        }

        // Add this request
        self.minute_requests.push(now);
        self.day_requests.push(now);

        true
    }

    fn cleanup_expired_requests(&mut self, now: Instant) {
        let minute_cutoff = now - Duration::from_secs(60);
        let day_cutoff = now - Duration::from_secs(24 * 60 * 60);

        self.minute_requests.retain(|&time| time > minute_cutoff);
        self.day_requests.retain(|&time| time > day_cutoff);
    }

    /// Get current usage stats
    fn get_usage(&self, now: Instant) -> (usize, usize) {
        let minute_cutoff = now - Duration::from_secs(60);
        let day_cutoff = now - Duration::from_secs(24 * 60 * 60);

        let minute_count = self
            .minute_requests
            .iter()
            .filter(|&&time| time > minute_cutoff)
            .count();

        let day_count = self
            .day_requests
            .iter()
            .filter(|&&time| time > day_cutoff)
            .count();

        (minute_count, day_count)
    }
}

/// In-memory rate limiter
#[derive(Debug)]
pub struct RateLimiter {
    trackers: Arc<RwLock<HashMap<String, RequestTracker>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            trackers: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Check if a request from the given key is allowed
    pub async fn check_request(&self, key: &str) -> Result<(), RateLimitError> {
        let mut trackers = self.trackers.write().await;
        let tracker = trackers
            .entry(key.to_string())
            .or_insert_with(RequestTracker::new);

        if tracker.check_and_add_request(&self.config) {
            Ok(())
        } else {
            let now = Instant::now();
            let (minute_usage, day_usage) = tracker.get_usage(now);

            if minute_usage >= self.config.requests_per_minute as usize {
                Err(RateLimitError::MinuteLimit {
                    limit: self.config.requests_per_minute,
                    current: minute_usage as u32,
                })
            } else {
                Err(RateLimitError::DayLimit {
                    limit: self.config.requests_per_day,
                    current: day_usage as u32,
                })
            }
        }
    }

    /// Get current usage stats for a key
    pub async fn get_usage(&self, key: &str) -> (u32, u32) {
        let trackers = self.trackers.read().await;
        if let Some(tracker) = trackers.get(key) {
            let now = Instant::now();
            let (minute_usage, day_usage) = tracker.get_usage(now);
            (minute_usage as u32, day_usage as u32)
        } else {
            (0, 0)
        }
    }

    /// Periodic cleanup of expired trackers
    pub async fn cleanup_expired_trackers(&self) {
        let mut trackers = self.trackers.write().await;
        let now = Instant::now();

        trackers.retain(|_, tracker| {
            let (minute_usage, day_usage) = tracker.get_usage(now);
            // Keep trackers that have recent activity
            minute_usage > 0 || day_usage > 0
        });
    }
}

/// Rate limiting errors
#[derive(Debug)]
pub enum RateLimitError {
    MinuteLimit { limit: u32, current: u32 },
    DayLimit { limit: u32, current: u32 },
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        let (status, message, limit, current) = match self {
            RateLimitError::MinuteLimit { limit, current } => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded: too many requests per minute",
                limit,
                current,
            ),
            RateLimitError::DayLimit { limit, current } => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded: too many requests per day",
                limit,
                current,
            ),
        };
        let body = Json(json!({
            "error": message,
            "limit": limit,
            "current": current,
            "retry_after": "60"
        }));
        (status, body).into_response()
    }
}

/// Extract client IP from request
pub fn extract_client_ip(headers: &HeaderMap, remote_addr: Option<SocketAddr>) -> IpAddr {
    // Try X-Forwarded-For header first
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(ip_str) = forwarded_str.split(',').next() {
                if let Ok(ip) = ip_str.trim().parse::<IpAddr>() {
                    return ip;
                }
            }
        }
    }

    // Try X-Real-IP header
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                return ip;
            }
        }
    }

    // Fall back to remote address
    if let Some(addr) = remote_addr {
        return addr.ip();
    }

    // Fallback to localhost
    IpAddr::from([127, 0, 0, 1])
}

/// Rate limiting middleware that handles both API key authenticated and public requests
pub async fn public_rate_limit_middleware(request: Request, next: Next) -> Result<Response, Response> {
    let remote_addr = request.extensions().get::<SocketAddr>().copied();
    let headers = request.headers().clone();

    // Extract IP for rate limiting
    let client_ip = extract_client_ip(&headers, remote_addr);

    // Apply rate limiting if rate limiter is available
    if let Some(limiter) = request.extensions().get::<Arc<RateLimiter>>().cloned() {
        if let Err(rate_limit_error) = limiter.check_request(&client_ip.to_string()).await {
            return Err(rate_limit_error.into_response());
        }
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
            requests_per_day: 10,
        };
        let limiter = RateLimiter::new(config);

        // First two requests should succeed
        assert!(limiter.check_request("test_ip").await.is_ok());
        assert!(limiter.check_request("test_ip").await.is_ok());

        // Third request should fail (minute limit)
        assert!(limiter.check_request("test_ip").await.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_different_ips() {
        let config = RateLimitConfig {
            requests_per_minute: 1,
            requests_per_day: 10,
        };
        let limiter = RateLimiter::new(config);

        // Different IPs should have separate limits
        assert!(limiter.check_request("ip1").await.is_ok());
        assert!(limiter.check_request("ip2").await.is_ok());

        // Second request from same IP should fail
        assert!(limiter.check_request("ip1").await.is_err());
        assert!(limiter.check_request("ip2").await.is_err());
    }

    #[tokio::test]
    async fn test_extract_client_ip() {
        use axum::http::HeaderMap;
        use std::net::{IpAddr, SocketAddr};

        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "192.168.1.1, 10.0.0.1".parse().unwrap());

        let ip = extract_client_ip(&headers, None);
        assert_eq!(ip, IpAddr::from([192, 168, 1, 1]));

        // Test with socket addr fallback
        let socket_addr = SocketAddr::from(([10, 0, 0, 1], 8080));
        let ip = extract_client_ip(&HeaderMap::new(), Some(socket_addr));
        assert_eq!(ip, IpAddr::from([10, 0, 0, 1]));
    }
}
