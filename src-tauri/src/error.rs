use std::fmt;

use polars::prelude::PolarsError;
use thiserror::Error;

/// Main error type for the QuakeTracker application
#[derive(Error, Debug)]
pub enum QuakeTrackerError {
    /// Network-related errors (HTTP requests, timeouts, connections)
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// JSON parsing errors
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// GeoJSON parsing errors  
    #[error("GeoJSON parsing error: {0}")]
    GeoJson(#[from] geojson::Error),

    /// Date/time parsing errors
    #[error("Date/time parsing error: {0}")]
    DateTime(#[from] chrono::ParseError),

    /// Analytics computation errors (Polars operations)
    #[error("Analytics error: {0}")]
    Analytics(#[from] PolarsError),

    /// I/O and storage errors
    #[error("Storage error: {0}")]
    Storage(#[from] std::io::Error),

    /// State management errors (locking, concurrency)
    #[error("State error: {0}")]
    State(String),

    /// Data validation errors
    #[error("Validation error: {field}: {message}")]
    Validation { field: String, message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    /// External service errors (EMSC API, WebSocket)
    #[error("External service error: {service}: {message}")]
    ExternalService { service: String, message: String },

    /// Resource exhaustion errors (memory, CPU)
    #[error("Resource exhaustion: {resource}: {message}")]
    ResourceExhaustion { resource: String, message: String },

    /// Internal application errors
    #[error("Internal error: {message}")]
    Internal { message: String },
}

/// Custom From implementation for PoisonError since it's generic
impl<T> From<std::sync::PoisonError<T>> for QuakeTrackerError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Self::State(format!("Lock poisoned: {}", err))
    }
}

impl QuakeTrackerError {
    /// Create a state error
    pub fn state<S: Into<String>>(message: S) -> Self {
        Self::State(message.into())
    }

    /// Create a validation error
    pub fn validation<S: Into<String>, T: Into<String>>(field: S, message: T) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create an external service error
    pub fn external_service<S: Into<String>, T: Into<String>>(service: S, message: T) -> Self {
        Self::ExternalService {
            service: service.into(),
            message: message.into(),
        }
    }

    /// Create a resource exhaustion error
    pub fn resource_exhaustion<S: Into<String>, T: Into<String>>(resource: S, message: T) -> Self {
        Self::ResourceExhaustion {
            resource: resource.into(),
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Network(_) => true,
            Self::ExternalService { .. } => true,
            Self::ResourceExhaustion { .. } => true,
            Self::State(_) => true,
            _ => false,
        }
    }

    /// Get the error category for logging/metrics
    pub fn category(&self) -> &'static str {
        match self {
            Self::Network(_) => "network",
            Self::Json(_) => "json",
            Self::GeoJson(_) => "geojson",
            Self::DateTime(_) => "datetime",
            Self::Analytics(_) => "analytics",
            Self::Storage(_) => "storage",
            Self::State(_) => "state",
            Self::Validation { .. } => "validation",
            Self::Configuration { .. } => "configuration",
            Self::ExternalService { .. } => "external_service",
            Self::ResourceExhaustion { .. } => "resource_exhaustion",
            Self::Internal { .. } => "internal",
        }
    }

    /// Get the severity level of this error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Network(_) => ErrorSeverity::Medium,
            Self::Json(_) | Self::GeoJson(_) | Self::DateTime(_) => ErrorSeverity::Low,
            Self::Analytics(_) => ErrorSeverity::Medium,
            Self::Storage(_) => ErrorSeverity::High,
            Self::State(_) => ErrorSeverity::High,
            Self::Validation { .. } => ErrorSeverity::Low,
            Self::Configuration { .. } => ErrorSeverity::High,
            Self::ExternalService { .. } => ErrorSeverity::Medium,
            Self::ResourceExhaustion { .. } => ErrorSeverity::Critical,
            Self::Internal { .. } => ErrorSeverity::Critical,
        }
    }
}

/// Error severity levels for monitoring and alerting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

pub type Result<T> = std::result::Result<T, QuakeTrackerError>;

/// Error context for better error reporting
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub component: String,
    pub additional_info: Option<String>,
}

impl ErrorContext {
    pub fn new<S: Into<String>, T: Into<String>>(operation: S, component: T) -> Self {
        Self {
            operation: operation.into(),
            component: component.into(),
            additional_info: None,
        }
    }

    pub fn with_info<S: Into<String>>(mut self, info: S) -> Self {
        self.additional_info = Some(info.into());
        self
    }
}

/// Extension trait for adding context to errors
pub trait ErrorContextExt<T> {
    fn with_context(self, context: ErrorContext) -> Result<T>;
    fn with_operation<S: Into<String>>(self, operation: S, component: S) -> Result<T>;
}

impl<T, E> ErrorContextExt<T> for std::result::Result<T, E>
where
    E: Into<QuakeTrackerError>,
{
    fn with_context(self, context: ErrorContext) -> Result<T> {
        self.map_err(|e| {
            let error = e.into();
            let context_str = format!("[{}:{}]", context.component, context.operation);
            let info_str = context.additional_info
                .map(|info| format!(" ({})", info))
                .unwrap_or_default();
            
            // For errors that contain the original error, we can't easily modify them
            // So we'll create new internal errors with context
            match error {
                QuakeTrackerError::State(msg) => {
                    QuakeTrackerError::State(format!("{} {}{}", context_str, msg, info_str))
                }
                other => {
                    QuakeTrackerError::Internal {
                        message: format!("{} {}{}", context_str, other, info_str)
                    }
                }
            }
        })
    }

    fn with_operation<S: Into<String>>(self, operation: S, component: S) -> Result<T> {
        self.with_context(ErrorContext::new(operation, component))
    }
}

/// Validation helper functions
pub mod validation {
    use super::*;

    pub fn validate_magnitude(magnitude: f64) -> Result<()> {
        if magnitude < -2.0 || magnitude > 10.0 {
            return Err(QuakeTrackerError::validation(
                "magnitude",
                format!("Magnitude {} is outside valid range [-2.0, 10.0]", magnitude),
            ));
        }
        Ok(())
    }

    pub fn validate_depth(depth: f64) -> Result<()> {
        if depth < 0.0 || depth > 700.0 {
            return Err(QuakeTrackerError::validation(
                "depth",
                format!("Depth {} is outside valid range [0.0, 700.0] km", depth),
            ));
        }
        Ok(())
    }

    pub fn validate_latitude(latitude: f64) -> Result<()> {
        if latitude < -90.0 || latitude > 90.0 {
            return Err(QuakeTrackerError::validation(
                "latitude",
                format!("Latitude {} is outside valid range [-90.0, 90.0]", latitude),
            ));
        }
        Ok(())
    }

    pub fn validate_longitude(longitude: f64) -> Result<()> {
        if longitude < -180.0 || longitude > 180.0 {
            return Err(QuakeTrackerError::validation(
                "longitude",
                format!("Longitude {} is outside valid range [-180.0, 180.0]", longitude),
            ));
        }
        Ok(())
    }

    pub fn validate_event_id(id: &str) -> Result<()> {
        if id.is_empty() {
            return Err(QuakeTrackerError::validation(
                "id",
                "Event ID cannot be empty",
            ));
        }
        if id.len() > 100 {
            return Err(QuakeTrackerError::validation(
                "id",
                format!("Event ID too long: {} characters (max 100)", id.len()),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = QuakeTrackerError::state("Connection failed");
        assert_eq!(error.category(), "state");
        assert_eq!(error.severity(), ErrorSeverity::High);
        assert!(error.is_recoverable());
    }

    #[test]
    fn test_error_context() {
        let result: Result<()> = Err(QuakeTrackerError::state("Test error"));
        let with_context = result.with_operation("test_operation", "test_component");
        
        assert!(with_context.is_err());
        let error = with_context.unwrap_err();
        assert!(error.to_string().contains("[test_component:test_operation]"));
    }

    #[test]
    fn test_validation() {
        assert!(validation::validate_magnitude(5.0).is_ok());
        assert!(validation::validate_magnitude(15.0).is_err());
        
        assert!(validation::validate_latitude(45.0).is_ok());
        assert!(validation::validate_latitude(95.0).is_err());
        
        assert!(validation::validate_longitude(120.0).is_ok());
        assert!(validation::validate_longitude(200.0).is_err());
    }

    #[test]
    fn test_error_conversion() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_error.is_err());
        
        let qt_error: QuakeTrackerError = json_error.unwrap_err().into();
        assert_eq!(qt_error.category(), "json");
    }
} 