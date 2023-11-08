# Associated Functions

Normally you have to define associated functions esternally:

```ruby
struct MyType {...}

function MyType::foo() -> Self {...}
```

This is fine, and looks nice, but if we wanted to construct a type in a const function, we would not be able to also define the associated functions.

If we allow writing associated functions inside the type body, like in Rust and Zig, we could write convenient constructors.

```ruby
const function MyType() -> type {
    return struct {
        ...

        function foo() -> Self {
            ...
        }
    }
}
```

We can use partial application to specialize a generic function.

```ruby
function add(a: int, b: int) -> int {
    return a + b
}

const add5 := add(_, 5)

print(add5(1))
# 6
```

This is convenient for specializing a type.

```ruby
import stl::collections::array

function Matrix(const ROWS: uint, const COLS: uint, const T: type) -> type {
    return array(ROWS, array(COLS, T))
}

const RowVector := Matrix(1, _, _)
const ColVector := Matrix(_, 1, _)
const FloatVector := RowVector(_, float)
const Vec3 := FloatVector(3)
```
