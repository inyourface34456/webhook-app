use crate::{Outer, to_outer};
use rand::{distributions::Alphanumeric, Rng};
use std::collections::HashMap;
use tokio::sync::broadcast::{self, Receiver, Sender};

#[derive(Clone)]
pub struct WebhookList{
    ids: Outer<HashMap<String, Sender<String>>>
}

impl WebhookList {
    pub fn new() -> Self {
        Self {
            ids: to_outer(HashMap::new())
        }
    }

    pub fn issue_id(&self) -> String {
        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(128)
            .map(char::from)
            .collect();

        if let Ok(mut ids) = self.ids.write() {
            let (tx, _rx) = broadcast::channel(16);

            ids.insert(s.clone(), tx);
        } 

        s
    }

    pub fn get_id(&self, id: String) -> Option<(Sender<String>, Receiver<String>)> {
        if id.len() != 128 {
            return None
        }

        if let Ok(ids) = self.ids.read() {
            match ids.get(&id) {
                Some(dat) => Some((dat.clone(), dat.subscribe())),
                None => None
            }
        } else {
            None
        }
    }
}