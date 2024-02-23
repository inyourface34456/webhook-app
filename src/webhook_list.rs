use crate::{Outer, to_outer};
use rand::{distributions::Alphanumeric, Rng};
use std::io::Read;
use std::{collections::HashMap, io::Write};
use std::fs;
use tokio::sync::broadcast::{self, Receiver, Sender};

#[derive(Clone)]
pub struct WebhookList{
    ids: Outer<HashMap<String, Sender<String>>>,
    path: String,
}

impl WebhookList {
    pub fn new(path: String) -> Self {
        Self {
            ids: to_outer(HashMap::new()),
            path,
        }
    }

    pub fn load(path: String) -> Self {
        if fs::metadata(&path).is_err() {
            Self::new(path)
        } else {
            match fs::File::open(&path) {
                Ok(mut dat) => {
                    let mut buf = String::new();
                    match dat.read_to_string(&mut buf) {
                        Ok(_) => {
                            let ids: Vec<String> = buf.split('\n').map(|x| x.into()).collect();
                            let mut map = HashMap::new();
                            for i in ids {
                                map.insert(i, broadcast::channel(16).0);
                            }
                            Self {ids: to_outer(map), path}
                        },
                        Err(_) => {
                            eprintln!("error reading ids.txt");
                            Self::new(path)
                        }
                    }
                },
                Err(_) => {
                    let _ = fs::File::create(&path);
                    Self::new(path)
                }
            }
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

    pub fn issue_perm_id(&self) -> Result<String, std::io::Error> {
        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(128)
            .map(char::from)
            .collect::<String>();

        if let Ok(mut ids) = self.ids.write() {
            let (tx, _rx) = broadcast::channel(16);

            ids.insert(s.clone(), tx);
        } 

        match fs::metadata(&self.path) {
            Ok(_) => {
                let mut file = fs::OpenOptions::new()
                    .append(true)
                    .open(&self.path)?;
                file.write_all(format!("{}\n", s).as_bytes())?;
                Ok(s.into())
            },
            Err(_) => {
                let mut file = fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(&self.path)?;
                file.write_all(format!("{}\n", s).as_bytes())?;
                Ok(s.into())
            }
        }
    }

    pub fn get_id(&self, id: String) -> Option<(Sender<String>, Receiver<String>)> {
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