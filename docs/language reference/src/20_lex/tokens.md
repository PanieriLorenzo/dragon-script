# Tokens

## Whitespace

Most unicode [Pattern_White_Space](https://www.unicode.org/reports/tr31/) characters do not have any special meaning and can be used interchangeably, with the exception of the newline character `U+0085` (`\n`).

In particular, these character should all be tokenized as `Ignore` tokens:

- `U+0009` (horizontal tab, `\t`)
- `U+000A` (line feed, `\n`)
- `U+000B` (vertical tab)
- `U+000C` (form feed)
- `U+000D` (carriage return, `\r`)
- `U+0020` (space, ` `)
- `U+200E` (left-to-right mark)
- `U+200F` (right-to-left mark)
- `U+2028` (line separator)
- `U+2029` (paragraph separator)

The newline character `U+0085` (`\n`) should be lexed as a `NewLine` token.
