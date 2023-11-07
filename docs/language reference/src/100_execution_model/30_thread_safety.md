# Thread Safety

Dragon provides some guarantees of thread safety, but not complete thread safety. In particular, it does not guarantee:
- Deadlock-safety
- Thread safety across FFI
- Thread safety when a program contains undefined behavior

## Sending Data Across Threads

By default, data can be safely sent across threads. Sending means passing a value to a thread when it is initially invoked, and reading the returned value once it exits.

This is how values are sent:

```ruby
import std::thread::Thread

function print_from_thread!(msg: str) -> {
    print!(msg)
}

printer := Thread::new(-> print_from_thread!("Hello!")).run()
printer.await()
```

## Sharing Data Across Threads

By default, values cannot be shared across threads. Sharing means capturing and accessing non-thread-local data.

The standard library provides types for sharing data across threads, such as `Mutex`, `Channel`, `Atomic`, `Semaphore` and `Lock`

These types are implemented in terms of a few intrinsics:

- `core::thread::__atomic_lock`: an atomic boolean flag, it can either be `locked` or `unlocked`, analogous to a C mutex.
- `core::thread::__atomic_counter`: an atomic integer counter, analogous to a C semaphore.

Types aren't inherently share-safe, they must implement the `Share` marker trait. This is also automatically implemented by types whose component types all implement `Share`. Implementing this trait explicitly raises a warning, unless we disable it explicitly, to avoid accidentally making a type shareable. The `Share` trait indicates that all safe modes of access to the type are share-safe.

```php
# core::marker::Share

trait Share {
    const __SHARE: none
}
```

For example, an over-simplified version of the `Atomic` type could look like this:

```ruby
struct Atomic[T] {
    _lock: core::thread::__atomic_lock
    _data: Cell[T]
}

# this is sufficient to implement the Share trait
const Atomic::__SHARE := none

# note the type-spaced function
function Atomic[T]::new(val: T) -> Self {
    return {
        _lock := core::thread::__atomic_lock::new()
        _data := Cell::new(val)
    }
}

function read![T](self: Atomic[T]) -> T {
    self._lock.acquire!()
    ret := self._data.read()
    self._lock.release!()
    return ret
}

# note that this doesn't need to mutate self because of Cell[T] providing interior mutability
function write![T](self: Atomic[T], val: T) {
    self._lock.acquire!()
    self._data.write(val)
    self._lock.release!()
}
```
