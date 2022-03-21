# zip-eq
A zip iterator that checks that its inputs have the same lengths, either eagerly at the moment of construction, or lazily.

# Examples

Eager check
```rust
use zip_eq::ZipEq;
                                         
let a = [1, 2];
let b = [3, 4];
let mut zipped = a.zip_eq_eager(b); // length check happens here
                                         
assert_eq!(zipped.next(), Some((1, 3)));
assert_eq!(zipped.next(), Some((2, 4)));
assert_eq!(zipped.next(), None);
```
Lazy check
```rust
use zip_eq::ZipEq;
                                         
let a = [1, 2];
let b = [3, 4];
let mut zipped = a.zip_eq_lazy(b);
                                         
assert_eq!(zipped.next(), Some((1, 3)));
assert_eq!(zipped.next(), Some((2, 4)));
assert_eq!(zipped.next(), None); // length check happens here
```
