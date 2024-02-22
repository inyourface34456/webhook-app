mod endpoint_funcs;
mod utils;
mod webhook_list;

use webhook_list::*;
use utils::*;
use endpoint_funcs::*;
use futures_util::StreamExt;
use tokio::sync::broadcast::{self, Receiver};
use tokio_stream::wrappers::BroadcastStream;
use warp::Filter;

#[tokio::main]
async fn main() {
    let ids = WebhookList::new();
    let ids_filter = warp::any().map(move || ids.clone());

    let routes = warp::path!("webhook" / String / "listen").and(warp::post()).and(ids_filter.clone()).map(move |id, id_list: WebhookList| { 
        let id_exist;
        let rx2: Option<Receiver<String>> = match id_list.get_id(id) {
            Some(dat) => Some(dat.1),
            None => None
        };        

        let stream = match rx2 {
            Some(rx2) => {
                id_exist = true;
                BroadcastStream::new(rx2)
            },
            None => {
                id_exist = false;
                BroadcastStream::new(broadcast::channel(16).1)
            }
            
        };

        let event_stream = stream.map(move |x| {
            if id_exist {
                match x {
                    Ok(x) => sse_counter(x),
                    Err(err) => sse_counter(err.to_string())
                }
            } else {
                sse_counter("not found".into())
            }
        });

        warp::sse::reply(event_stream)
    });

    let send_to_data = warp::post()
        .and(warp::path!("webhook" / String / "send_text"))
        .and(warp::path::end())
        .and(json_string())
        .and(ids_filter.clone())
        .and_then(send);

    let get_id = warp::post()
        .and(warp::path("get_id"))
        .and(warp::path::end())
        .and(ids_filter.clone())
        .and_then(issue_id);

    let route = send_to_data.or(routes).or(get_id);

    warp::serve(route).run(([127, 0, 0, 1], 3030)).await;
}
