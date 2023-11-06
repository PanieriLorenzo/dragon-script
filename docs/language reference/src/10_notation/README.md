# Notation

Syntax is documented using Extended Backus-Nauer Form (EBNF) and railroad diagrams.

## Node Types
The leaf nodes of the EBNF rules can be strings, regular expressions or other EBNF rules.

```rs
Regex ::= [a-z]+
Rule ::= Foo
Literals ::= 'foo' "bar"
```

![regex](./regex.svg)

![rule](./rule.svg)

![literals](./literals.svg)

## Quantifiers

```rs
one ::= foo
zero_or_one ::= foo?
one_or_more ::= foo+
zero_or_more ::= foo*
```

![one](./one.svg)

![zero_or_one](./zero_or_one.svg)

![one_or_more](./one_or_more.svg)

![zero_or_more](./zero_or_more.svg)

## Expressions

```rs
alternation ::= foo | bar
concatenation ::= foo bar
group ::= (foo | bar) baz
```

![alternation](./alternation.svg)

![concatenation](./concatenation.svg)

![group](./group.svg)
