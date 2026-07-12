# crossbeam-channel 0.5.14 API Summary

## Key Functions
- `bounded(cap: usize) -> (Sender<T>, Receiver<T>)`
- `unbounded() -> (Sender<T>, Receiver<T>)`

## Sender<T>
- `send(&self, msg: T) -> Result<(), SendError<T>>`
- `try_send(&self, msg: T) -> Result<(), TrySendError<T>>`
- `send_timeout(&self, msg: T, dur: Duration) -> Result<(), SendTimeoutError<T>>`
- `is_empty(&self) -> bool`
- `is_full(&self) -> bool`
- `len(&self) -> usize`
- `capacity(&self) -> Option<usize>`
- Implements Clone (multi-producer)

## Receiver<T>
- `recv(&self) -> Result<T, RecvError>`
- `try_recv(&self) -> Result<T, TryRecvError>`
- `recv_timeout(&self, dur: Duration) -> Result<T, RecvTimeoutError>`
- `is_empty(&self) -> bool`
- `is_full(&self) -> bool`
- `len(&self) -> usize`
- `capacity(&self) -> Option<usize>`
- `iter(&self) -> Iter<T>` — blocking iterator
- `try_iter(&self) -> TryIter<T>` — non-blocking iterator
- Implements Clone (multi-consumer)
