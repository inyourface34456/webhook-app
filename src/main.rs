mod endpoint_funcs;
mod utils;
mod webhook_list;

use std::{net::{SocketAddrV4, SocketAddrV6, SocketAddr}, str::FromStr};

use webhook_list::*;
use utils::*;
use endpoint_funcs::*;
use futures_util::StreamExt;
use tokio::sync::broadcast::{self, Receiver};
use tokio_stream::wrappers::BroadcastStream;
use warp::Filter;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about)]
struct Args {
    /// file to load webhook ID's from (seperated by newlines).  If the file does not exist, this will create it.
    #[arg(long = "load-name", short = 'f', default_value_t = String::from("ids.txt"))]
    load_name: String,

    /// weather to use ip v4 vs v6
    #[arg(long = "ipv6", short = '6', default_value_t = false)]
    ipv6: bool,

    /// Adress to bind to
    #[arg(long = "address", short = 'a', default_value_t = String::from("127.0.0.1:3030"))]
    address: String,

}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let ids = WebhookList::load(args.load_name);
    let ids_filter = warp::any().map(move || ids.clone());

    let route_post = warp::path!("webhook" / String / "listen").and(warp::post()).and(ids_filter.clone()).map(move |id, id_list: WebhookList| { 
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

    let route_get = warp::path!("webhook" / String / "listen").and(warp::post()).and(ids_filter.clone()).map(move |id, id_list: WebhookList| { 
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
        .and(warp::path!("webhook" / String / "send"))
        .and(warp::path::end())
        .and(json_string())
        .and(ids_filter.clone())
        .and_then(send);

    let get_id = warp::post()
        .and(warp::path("issue_id"))
        .and(warp::path::end())
        .and(ids_filter.clone())
        .and_then(issue_id);

    let get_perm_id = warp::post()
        .and(warp::path("issue_perm_id"))
        .and(warp::path::end())
        .and(ids_filter.clone())
        .and_then(issue_perm_id);

    let route = send_to_data.or(route_get).or(route_post).or(get_id).or(get_perm_id);
    let address: SocketAddr;


    if args.ipv6 {
        address = match SocketAddrV6::from_str(&args.address) {
            Ok(dat) => dat.into(),
            Err(err) => {
                eprintln!("{}", err.to_string());
                std::process::exit(1);
            }
        };
    } else {
        address = match SocketAddrV4::from_str(&args.address) {
            Ok(dat) => dat.into(),
            Err(err) => {
                eprintln!("{}", err.to_string());
                std::process::exit(1);
            }
        };
    }

    warp::serve(route).run(address).await;
}
