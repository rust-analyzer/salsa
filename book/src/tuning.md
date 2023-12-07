# Tuning Salsa

## LRU Cache

You can specify an LRU cache size for any non-input query:

```rs
let lru_capacity: usize = 128;
base_db::ParseQuery.in_db_mut(self).set_lru_capacity(lru_capacity);
```

The default is `0`, which disables LRU-caching entirely.

See [The LRU RFC for more details](./rfcs/RFC0004-LRU.md).

Note that there is no garbage collection for keys and
results of old queries, so LRU caches are currently the
only knob available for avoiding unbounded memory usage
for long-running apps built on Salsa.

## Intern Queries

Intern queries can make key lookup cheaper, save memory, and
avoid the need for [`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html).

Interning is especially useful for queries that involve nested,
tree-like data structures.

See:
- The [Intern Queries RFC](./rfcs/RFC0002-Intern-Queries.md)
- The [`compiler` example](https://github.com/salsa-rs/salsa/blob/master/examples/compiler/main.rs),
which uses interning.

## Granularity of Incrementality

See:
- [common patterns: selection](./common_patterns/selection.md) and
- The [`selection` example](https://github.com/salsa-rs/salsa/blob/master/examples/selection/main.rs)

## Cancellation

Queries that are no longer needed due to concurrent writes or changes in dependencies are cancelled
by Salsa. Each accesss of an intermediate query is a potential cancellation point. cancellation is
implemented via panicking, and Salsa internals are intended to be panic-safe.

If you have a query that contains a long loop which does not execute any intermediate queries,
salsa won't be able to cancel it automatically. You may wish to check for cancellation yourself
by invoking `db.unwind_if_cancelled()`.

For more details on cancellation, see:
- [the Opinionated cancellation RFC](./rfcs/RFC0007-Opinionated-Cancelation.md).
- The tests for cancellation behavior in the Salsa repo.

