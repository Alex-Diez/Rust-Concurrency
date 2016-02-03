pub use self::array_queue::ArrayBlockingQueue;
pub use self::linked_queue::UnboundedBlockingQueue;

mod array_queue;
mod linked_queue;

pub trait BlockingQueue<T> {
    
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn enqueue(&self, e: T);

    fn dequeue(&self) -> T;

    fn contains(&self, e: T) -> bool;

    fn offer(&self, e: T) -> bool;

    fn peek(&self) -> Option<T>;
}
