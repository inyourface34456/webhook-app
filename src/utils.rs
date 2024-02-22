use std::sync::{Arc, RwLock};
use warp::{body, Filter, Rejection};

pub fn json_string() -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    body::content_length_limit(1024 * 16).and(body::json())
}

pub type Outer<T> = Arc<RwLock<T>>;

pub fn to_outer<T>(data: T) -> Outer<T> {
    Arc::new(RwLock::new(data))
}