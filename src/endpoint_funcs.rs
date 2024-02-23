use warp::{Reply, reply, Rejection, sse::Event, reject};
use crate::WebhookList;
use std::convert::Infallible;

pub fn sse_counter(counter: String) -> Result<Event, Infallible> {
    Ok(warp::sse::Event::default().data(counter))
}

pub async fn send(id: String, data: String, ids: WebhookList) -> Result<impl Reply, Rejection> {
    let sender = match ids.get_id(id) {
        Some(dat) => dat.0,
        None => return Err(reject())
    };
    
    let _ = sender.send(data);
    Ok(reply::html("ok"))
}

pub async fn issue_id(ids: WebhookList) -> Result<impl Reply, Rejection> {
    let id = ids.issue_id();

    Ok(reply::json(&id))
}

pub async fn issue_perm_id(ids: WebhookList) -> Result<impl Reply, Rejection> {
    let id = match ids.issue_perm_id() {
        Ok(dat) => dat,
        Err(err) => return Ok(reply::with_status(reply::json(&err.to_string()), warp::http::StatusCode::INTERNAL_SERVER_ERROR))
    };

    Ok(reply::with_status(reply::json(&id), warp::http::StatusCode::OK))
}