use std::sync::{Arc, Condvar, Mutex};
use std::collections::VecDeque;

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}
impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders += 1;
        drop(inner);
        Sender {
            shared: Arc::clone(&self.shared),
        }
    }
}
impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders -= 1;
        let was_last = inner.senders == 0;
        drop(inner);
        if was_last {
            self.shared.condvar.notify_all();
        }
    }
}
pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
    buffer: VecDeque<T>,
}
impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Receiver {
            shared: Arc::clone(&self.shared),
            buffer: VecDeque::new(),
        }
    }
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.queue.push_back(value);
        // important to drop the lock before notifying the condition variable
        drop(inner);
        self.shared.condvar.notify_one(); 
    }
}

impl<T> Receiver<T> {
    pub fn try_recv(&self) -> Option<T> {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.queue.pop_front()
    }
    pub fn recv(&mut self) -> Option<T> {
        if let Some(t) = self.buffer.pop_front() {
            return Some(t);
        }
        let mut inner = self.shared.inner.lock().unwrap();
        loop {
            match inner.queue.pop_front() {
                Some(value) => {
                    std::mem::swap(&mut self.buffer, &mut inner.queue);
                    return Some(value);
                }
                None if inner.senders == 0 => return None,
                None => {
                    inner = self.shared.condvar.wait(inner).unwrap();
                }
            }
        }
    }
}
struct Shared<T> {
    inner: Mutex<Inner<T>>,
    condvar: Condvar,
}
struct Inner<T> {
    queue: VecDeque<T>,
    senders: usize,
}
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: VecDeque::new(),
        senders: 1,
    };
    let shared = Arc::new(Shared {
        inner: Mutex::new(inner),
        condvar: Condvar::new(),
    });
    (Sender { shared: shared.clone() }, Receiver { shared, buffer: VecDeque::new() })
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_channel() {
        let (sender, receiver) = channel();
        sender.send(42);
        assert_eq!(receiver.recv(), Some(42));
    }
    #[test]
    fn test_channel_closed() {
        let (tx, rx) = channel::<()>();
        drop(tx);
        assert_eq!(rx.recv(), None);
    }
}
