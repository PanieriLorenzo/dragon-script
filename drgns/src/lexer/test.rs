use strum::IntoEnumIterator;

use crate::{arena::intern, two_char_strings};

use super::{
    test_utils::{make_lexer, tokens_2_str},
    TokenType,
};

use itertools::iproduct;

#[test]
fn lex_single_tokens() {
    let mut lx = make_lexer();

    // actual test
    for tt in TokenType::iter() {
        intern(tokens_2_str(tt).to_string());
        intern(" ".to_string());
        assert_eq!(lx.next().unwrap().token_type, tt);
        assert_eq!(lx.next().unwrap().token_type, TokenType::Ignore);
    }
}

#[test]
fn lex_token_pairs() {
    let mut lx = make_lexer();

    for (tt1, tt2) in iproduct!(TokenType::iter(), TokenType::iter()) {
        intern(tokens_2_str(tt1).to_string());
        intern(" ".to_string());
        intern(tokens_2_str(tt2).to_string());
        intern(" ".to_string());
        assert_eq!(lx.next().unwrap().token_type, tt1);
        assert_eq!(lx.next().unwrap().token_type, TokenType::Ignore);
        assert_eq!(lx.next().unwrap().token_type, tt2);
        assert_eq!(lx.next().unwrap().token_type, TokenType::Ignore);
    }
}

#[test]
fn lex_arbitrary_text() {
    let lx = make_lexer();
    for s in two_char_strings!() {
        intern(s);
    }
    // just asserting that it lexes all the way through
    for _ in lx {}
}
