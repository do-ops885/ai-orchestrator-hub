use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rate limiter for protecting API endpoints and preventing abuse
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Maximum requests per window
    max_requests: u32,
    /// Time window for rate limiting
    window_duration: Duration,
    /// Storage for client request counts
    clients: Arc<RwLock<HashMap<String, ClientState>>>,
}

#[derive(Debug, Clone)]
struct ClientState {
    /// Number of requests in current window
    request_count: u32,
    /// Start time of current window
    window_start: Instant,
    /// Last request time for cleanup
    last_request: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter with specified limits
    ///
    /// # Arguments
    /// * `max_requests` - Maximum requests allowed per window
    /// * `window_duration` - Duration of the rate limiting window
    ///
    /// # Examples
    /// ```
    /// // Allow 100 requests per minute
    /// let limiter = RateLimiter::new(100, Duration::from_secs(60));
    /// ```
    pub fn new(max_requests: u32, window_duration: Duration) -> Self {
        Self {
            max_requests,
            window_duration,
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a request from the given client should be allowed
    ///
    /// # Arguments
    /// * `client_id` - Unique identifier for the client (IP, user ID, etc.)
    ///
    /// # Returns
    /// * `Ok(())` if request is allowed
    /// * `Err(RateLimitError)` if rate limit exceeded
    pub async fn check_rate_limit(&self, client_id: &str) -> Result<(), RateLimitError> {
        let mut clients = self.clients.write().await;
        let now = Instant::now();

        let client_state = clients.entry(client_id.to_string()).or_insert(ClientState {
            request_count: 0,
            window_start: now,
            last_request: now,
        });

        // Check if we need to reset the window
        if now.duration_since(client_state.window_start) >= self.window_duration {
            client_state.request_count = 0;
            client_state.window_start = now;
        }

        // Update last request time
        client_state.last_request = now;

        // Check rate limit
        if client_state.request_count >= self.max_requests {
            let reset_time = client_state.window_start + self.window_duration;
            return Err(RateLimitError {
                max_requests: self.max_requests,
                window_duration: self.window_duration,
                reset_time,
                current_count: client_state.request_count,
            });
        }

        // Increment request count
        client_state.request_count += 1;
        Ok(())
    }

    /// Get current rate limit status for a client
    pub async fn get_rate_limit_status(&self, client_id: &str) -> RateLimitStatus {
        let clients = self.clients.read().await;
        let now = Instant::now();

        if let Some(client_state) = clients.get(client_id) {
            let remaining = if now.duration_since(client_state.window_start) >= self.window_duration
            {
                self.max_requests // Window has reset
            } else {
                self.max_requests.saturating_sub(client_state.request_count)
            };

            let reset_time = client_state.window_start + self.window_duration;

            RateLimitStatus {
                max_requests: self.max_requests,
                remaining_requests: remaining,
                reset_time,
                window_duration: self.window_duration,
            }
        } else {
            RateLimitStatus {
                max_requests: self.max_requests,
                remaining_requests: self.max_requests,
                reset_time: now + self.window_duration,
                window_duration: self.window_duration,
            }
        }
    }

    /// Clean up old client entries to prevent memory leaks
    pub async fn cleanup_old_entries(&self) {
        let mut clients = self.clients.write().await;
        let now = Instant::now();
        let cleanup_threshold = Duration::from_secs(3600); // 1 hour

        clients.retain(|_, state| now.duration_since(state.last_request) < cleanup_threshold);
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                self.cleanup_old_entries().await;
            }
        });
    }
}

/// Rate limit error information
#[derive(Debug, Clone)]
pub struct RateLimitError {
    pub max_requests: u32,
    pub window_duration: Duration,
    pub reset_time: Instant,
    pub current_count: u32,
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Rate limit exceeded: {}/{} requests in {:?}. Reset in {:?}",
            self.current_count,
            self.max_requests,
            self.window_duration,
            self.reset_time.duration_since(Instant::now())
        )
    }
}

impl std::error::Error for RateLimitError {}

/// Current rate limit status for a client
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub max_requests: u32,
    pub remaining_requests: u32,
    pub reset_time: Instant,
    pub window_duration: Duration,
}

/// Predefined rate limiters for different use cases
pub struct RateLimiters {
    /// General API rate limiter (100 requests per minute)
    pub api: Arc<RateLimiter>,
    /// Agent creation rate limiter (10 per minute)
    pub agent_creation: Arc<RateLimiter>,
    /// Task creation rate limiter (50 per minute)
    pub task_creation: Arc<RateLimiter>,
    /// WebSocket connection rate limiter (5 per minute)
    pub websocket: Arc<RateLimiter>,
}

impl RateLimiters {
    /// Create new rate limiters with sensible defaults
    pub fn new() -> Self {
        let api = Arc::new(RateLimiter::new(100, Duration::from_secs(60)));
        let agent_creation = Arc::new(RateLimiter::new(10, Duration::from_secs(60)));
        let task_creation = Arc::new(RateLimiter::new(50, Duration::from_secs(60)));
        let websocket = Arc::new(RateLimiter::new(5, Duration::from_secs(60)));

        // Start cleanup tasks
        Arc::clone(&api).start_cleanup_task();
        Arc::clone(&agent_creation).start_cleanup_task();
        Arc::clone(&task_creation).start_cleanup_task();
        Arc::clone(&websocket).start_cleanup_task();

        Self {
            api,
            agent_creation,
            task_creation,
            websocket,
        }
    }
}

impl Default for RateLimiters {
    fn default() -> Self {
        Self::new()
    }
}
