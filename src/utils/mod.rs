use worker::*;

/// Logs information about an incoming request
pub fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().and_then(|cf| cf.coordinates()).unwrap_or_default(),
        req.cf().and_then(|cf| cf.region()).unwrap_or_else(|| "unknown region".into())
    );
}
