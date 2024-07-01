use std::{collections::HashMap, sync::Arc};

use chia::protocol::Message;
use tokio::sync::{oneshot, Mutex, OwnedSemaphorePermit, Semaphore};

#[derive(Debug)]
pub(super) struct Request {
    sender: oneshot::Sender<Message>,
    _permit: OwnedSemaphorePermit,
}

impl Request {
    pub(super) fn send(self, message: Message) {
        self.sender.send(message).ok();
    }
}

#[derive(Debug)]
pub(super) struct Requests {
    items: Mutex<HashMap<u16, Request>>,
    semaphore: Arc<Semaphore>,
}

impl Requests {
    pub(super) fn new() -> Self {
        Self {
            items: Mutex::new(HashMap::new()),
            semaphore: Arc::new(Semaphore::new(u16::MAX as usize)),
        }
    }

    pub(super) async fn insert(&self, sender: oneshot::Sender<Message>) -> u16 {
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("semaphore closed");

        let mut items = self.items.lock().await;

        let mut index = None;

        for i in 0..=u16::MAX {
            if !items.contains_key(&i) {
                index = Some(i);
            }
        }

        let index = index.expect("exceeded expected number of requests");
        items.insert(
            index,
            Request {
                sender,
                _permit: permit,
            },
        );
        index
    }

    pub(super) async fn remove(&self, id: u16) -> Option<Request> {
        self.items.lock().await.remove(&id)
    }
}
