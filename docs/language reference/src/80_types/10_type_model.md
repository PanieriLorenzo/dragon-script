# Type Model

Dragon uses algebraic data types (ADTs) as well as composite datatypes such as types, objects and lists as the basis for its type system.

Types are formally modelled as sets of values. For instance, `int` is the set of every signed integers that fit in 64 bits using 2's complement. New types can be composed from old types by algebraic manipulation of sets, as well as by special mappings called *type combinators*.

Types are first-class members of the language, so they must be representable in the language as [regular values](../100_execution_model/10_values.md). For this reason, the language is equipped with a `type` type. This is a [meta-type](#meta-types), in other words, a type whose members are also types. This meta-type is defined as the set of all types. This set is recursive, as it includes itself.

## First-Order Types

First-order types are typical types, that are modelled as sets of values.

There are two types that are considered "atomic" as they cannot be broken down further.

- `never`: a type that has no members. As `never` has no members, it cannot be instantiated at any point, and is used to model impossible scenarios, for example, it's the return type of a function that always panics.
- `none`: the unit type. It has only one member, we don't care about the value of its member, in fact we use the keyword `none` interchangeably to mean the type `none` and the value `none`. Note that many unit-types are contypeable other than `none`, but we tend to only model `none` as having all the properties of the unit type in the type-theoretical sense.

We can contype a type directly from its members, since values aren't types, we cannot use type operators on them, instead we have to leapfrog through the `set` type first, as sets can be easily converted to types.

```ruby
DecimalDigit := {0, 1, 2, 3, 4, 5, 6, 7, 8, 9}.to_type()
```

We will define the `set` type later, when we talk about composite types, for now just assume that we can use it to contype basic types.

The language provides us with some built-in types. These are colloquially known as *plain old data* (POD) types. These are defined in the compiler, so they don't have a formal definition expressed in terms of this type model, but we can approximately describe them in terms of their members.

```ruby
never := empty.to_type()

bool := {false, true}.to_type()
int := {0, 1, 2, 3, ..., -1, -2, -3, ...}.to_type()
uint := {0, 1, 2, 3, ...}.to_type()
float := {0.0, 0.1, 0.2, ..., -0.0, -0.1, -0.2, ..., nan, -nan, inf, -inf}.to_type()
```

The language additionally provides us with some more abstract types:
- `any`: the type defined as having all values (top type), including itself

Next, we need to define composite types, i.e. types that combine simpler types to contype more complex typeures. These are still first-order types, as their members are values, but they just have a more complex typeure.

The first two composite types are sum-types (enums) and product-types (tuples). These form the basis for contypeing algebraic data types (ADTs).

The sum-type (denoted with the operator `|`), is defined as the set of values resulting from the union of two types.

```ruby
TypeFoo := {1, 2, 3}.to_type()
TypeBar := {3, 4, 5}.to_type()
TypeFooBar := TypeFoo | TypeBar
assert!(TypeFooBar == {1, 2, 3, 4, 5}.to_type())
```

The sum-type is used very often to implement dynamic-like code, without explicitly requiring dynamic typing contypes. Very often we use sum-types to represent nullable data. We will colloquially call this pattern *option* types or *optional* types. We can write an imperative style deferred initialization like this:

```ruby
maybe_uninitialized: int | none := none
{
    # expensive initialization code
    ...
}

if maybe_uninitialized is int {
    println!(maybe_uninitialized)
}
```

Sum-types are *transparent*, in that there is no way to distinguish if a value is a member of a type or a member of a super-type. The summation operation, in other words, is referentially transparent.

```ruby
foo: int = 42
bar: none | int = 42

assert!(foo is int)
assert!(bar is int)
```

It's easy to define a sound `is` operator, but it's very tricky to define a sound function to extract the type of an operator. In general, the type of a value is relative to the context it is used in.

Product types are the combination of two types, such that all the members of the product are pairs of values resulting from the cartesian product of two types. We denote product types with the `,` operator. To disambiguate, in most context we are required to parenthesize the product type like this: `(int, float)`. Product types are often called *tuples*.

```ruby
TypeFoo := {1, 2}.to_type()
TypeBar := {'a', 'b'}.to_type()
TypeFooBar := (TypeFoo, TypeBar)
assert!(TypeFooBar == {(1, 'a'), (2, 'a'), (1, 'b'), (2, 'b')}.to_type())
```

Note that product types are not commutative, while sum types are. Both are associative.

```ruby
Foo := {1, 2}.to_type()
Bar := {'a', 'b'}.to_type()
Baz := {false, true}.to_type()

# commutativity
assert!(Foo | Bar == Bar | Foo)

# associativity
assert!((Foo | Bar) | Baz == Foo | (Bar | Baz))
assert!(((Foo, Bar), Baz) == (Foo, (Bar, Baz)))

# distributivity
assert!((Foo, Bar | Baz) == (Foo, Bar) | (Foo, Baz))
```

This is slightly different to other languages that follow a more imperative type system, like `Rust`, where tuples are not associative, and there aren't pure sum types (rust has enums, but they are not transparent, so they don't fully qualify).

Other than the basic type operators of ADTs, there are additional operators that will become very useful with traits.

There are intersection types. Intersection types represent types that share properties in common with two parent types, they are made of the intersection of their parent types. We notate them with the `&` operator.

```rust
SomeInts := {0, 1, 2, 3, 4, 5, 6}
EvenNumbers := {0, 0.0, 2, 2.0, 4, 4.0, 6, 6.0}
EvenInts := SomeInts & EvenNumbers
assert!(EvenInts == {0, 2, 4, 6}.to_type())
```

Finally there are negative types, which are abstract types that include all values except the ones in a set.

```rust
NotInt := !int
```

We can contype subtraction types fairly easily by combining negative and intersection types, like this:
```rust
SomeInts := {0, 1, 2, 3, 4, 5, 6}
EvenNumbers := {0, 0.0, 2, 2.0, 4, 4.0, 6, 6.0}
OddInts := SomeInts & !EvenNumbers
assert!(OddInts == {1, 3, 5}.to_type())
```

So far, the type operators we have seen are *transparent*. The operators can be replaced with the result, with no loss of information. Sometimes we don't want this, an example is the *newtype* pattern, that is very common in Rust.

Say we want to represent currencies, if we just defined two currencies like this, we would have problems:

```ruby
# NOTE: never use floats for money in real life! This is just an example
Dollars := float
Euros := float

dollars_balance: Dollars = 1000.0
euros_balance: Euros = 1000.0

# this is bad! The two should not be equivalent
assert!(dollars_balance == euros_balance)
```

Opaque types let us define two distinct types, that are both just wrappers around `float`, but which do not inherit the properties of their inner types. We can do this by wrapping a type in a `type` expression.

```rust
OpaqueSum := type(int | float)
OpaqueProduct := sturct(int, float)
```

We can now define types with equivalent signatures, which are not comparable.

```rust
TypeFoo := type(int)
TypeBar := type(int)
foo := TypeFoo(42)
bar := TypeBar(42)
assert!(foo != bar)
```

The language also provides an alternative syntax for types, where fields are labelled: this is the object notation, and it resembles classes in OOP languages. This is not required for a theoretical model of the language, it is just convenient.

```ruby
# labelled product type
Person := type {
    name: str,
    age: uint,
}

# labelled sum type
NaiveMoney := type {
    | dollars: float
    | euros: float
}

# a combination of the two
CorrectMoney := type {
    | dollars: {
        wholes: int,
        cents: int
    }
    | euros: {
        wholes: int,
        cents: int
    }
}
```

types also serve as a namespace, or more precisely a *typespace* for functions associated with the type, but this is beyond the type model.

Next, we need to define *contypeors*. Contypeors are just functions that retun a type. We can use them to model generic types. For instance, we can make our custom sum and product types:

```rust
function my_sum(a: type, b: type) -> type {
    return a | b
}

function my_product(a: type, b: type) -> type {
    return (a, b)
}
```

We use *contypeors* to model the remaining composite types. Note that these are defined in the compiler and there is no way to write a contypeor for them, we are just interested in the signature that such a contype would have

```ruby
# a list [T]
function list(T: type) -> type

# a set {T}
function set(T: type) -> type
```

The final layer of first-roder types are traits. Traits are types that are described in therms of their properties, rather than by enumerating their membders. Traits may have infinite members, so it's impossible to contype them from the bottom, we have to contype them from the top, by carving out a subset of the `any` type based on certain properties that we want the trait to have.

Traits are essentially a contract, with a set of rules that a type must obey to be a sub-type of the trait. The most straight-forward trait is one with a set of function signatures that the type must implement.

```rust
trait Divisible {
    function div(Self, Self) -> Self
    function rem(Self, Self) -> Self
}
```

Here `Self` is a place-holder type, that stands in for the sub-type of `Divisible` that the trait is being defined for.

Every type for which this function exists, is considered to be in the `Divisible` type.

Sometimes it is required that another trait must be implemented inorther to implement another trait. The type system allows us to do this with the tools we know so far.

```rust
trait Foo {
    function foo(Self & Bar) -> Self
}
```

Here, a type must implement `Bar` in order to also implemnt `Foo`.

This relation can be inverted, with the use of type-negation

```rust
trait Foo {
    function foo(Self & !Bar) -> Self
}
```

Here, a type must not implement `Bar`, in order to also implement `Foo`. This makes `Foo` and `Bar` mutually exclusive.

Some other constraints can be defined in traits, such as having specific associated constants:

```rust
trait One {
    const ONE: Self
}
```

Traits can be easily combined using the type-intersection operator:

```rust
Algebraic := Additive & Subtractive & Multiplicative & Divisive
```

With traits, we have reached the end of what first-order types can model. We can do a lot with this model, but there are still some features that cannot be implemented with first-order types alone.
- generics and polymorphism is still somewhat limited

## Higher-Order Types (Meta-Types)

Second order types are types whose members are first-order types. For example the type `{int, float}.to_type()` is not made up of all ints and floats, but instead it is literally made of the two types `int` and `float` in an abstract sense. So where `int` and `float` have many values, `{int, float}.to_type()` has only two values: `int` and `float`.

Third-order and higher types have similar definitions, but instead of being made of first-order types, they are made of second-order types, etc...

In general, any type that has other types as values, is called a `meta-type`, and we will be using this term instead of the specific order, because defining orders is a bit tricky and also not very useful. The language will thus only distinguish between first-order types and meta-types.

Most languages only have first-order types, but use special syntax to write generic and polymorphic code. Rust, for instance, uses a combination of traits and generic type parameters to allow flexible generic code. This works quite well, but leaves some sharp corners in the language, like the existence of generic const values side-by-side with const expressions. Additionally, generics generally require additional specialized syntax.

Languages like Zig don't have this issue, as they don't have special syntax of generics, they just use the type system to represent higher-order types. This works very well for Zig, as it doesn't have traits. But Zig only has a single meta-type, called `type`, so it cannot represent meta-types that only include specific types. This is not good enough for combining traits with generics. We need to have a more robust definition of `meta-type`.

We can define a trivial higher-order type by *lifting* a first-order type. This uses syntax that we have previously defined for [first-order types](#first-order-types)

```rust
N: {Numeric}.to_type()
```

This is already very useful, as we can use it to define a generic function like this:
```rust
function multiply_and_add(T: {Numeric}, a: T, b: T, c: T) -> T {
    return a * b + c
}
```

Note that we don't need to write the full `{...}.to_type()` in type signatures, the `.to_type()` part is implied.

There is no simple and portable way of defining such a function if we didn't have meta-types, we would be forced to introduce new syntax, like in Rust, where it would look something like this:

```rust
use num::Num;

fn multiply_an_add<T: Num>(a: T, b: T, c: T) -> T {
    a * b * c
}
```

This is okay, but we now need to know what `<...>` means, and that it has all kinds of special rules that do not resemble regular parameters. Ultimately it is a subjective design choice whether to use generic arguments or have meta-types. For Dragon, we wanted the language to have relatively little syntax.

Now, there are probably many contypes that we haven't thought about that can be contypeed using generic types, but the gist of it, is that we can write generic code, where we impose restrictions on which types the generic parameters can have.

Third-order and higher enters academic territory and doesn't have many practical uses.

## Proof of Soundness

- TODO: show Russel's paradox is not present
- TODO: show soundness of "is subtype of" and "type of" relations., hint: "type of" is probably unsound unless we introduce more stipulations, like "the type of a value is always the smallest opaque type that has it as one of its members". But this would leave cases were there are two candidate opaque types.
