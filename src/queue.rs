use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};
use lazy_static::lazy_static;

use crate::websocket_client::OpcUaData;

pub struct Queue<T> {
    data: Arc<Mutex<VecDeque<T>>>,
    capacity: usize,
    condvar: Arc<Condvar>, 
}

impl<T> Queue<T> {
    pub fn new(capacity: usize) -> Self {
        Queue {
            data: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
            condvar: Arc::new(Condvar::new()), 
        }
    }

    pub fn push(&self, item: T) -> Result<(), String> {
        let mut data: std::sync::MutexGuard<'_, VecDeque<T>> = self.data.lock().unwrap();
        
       
        while data.len() >= self.capacity {
            data = self.condvar.wait(data).unwrap(); 
        }

        data.push_back(item);
        
      
        self.condvar.notify_all();
        
        Ok(())
    }

    pub fn pop(&self) -> Option<T> {
        let mut data: std::sync::MutexGuard<'_, VecDeque<T>> = self.data.lock().unwrap();
        
       
        while data.is_empty() {
            data = self.condvar.wait(data).unwrap(); 
        }

        let item: Option<T> = data.pop_front();
        
       
        self.condvar.notify_all();
        
        item
    }

   
}

lazy_static! {
    pub static ref QUEUE: Arc<Queue<OpcUaData>> = Arc::new(Queue::new(20_000)); 
}