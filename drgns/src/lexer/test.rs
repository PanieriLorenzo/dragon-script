use std::rc::Rc;

use strum::IntoEnumIterator;

use crate::{
    eh::ErrorHandler,
    source::{Reader, SourceArena},
    two_char_strings,
};

use super::{test_utils::tokens_2_str, Lexer, TokenType};

use itertools::iproduct;

fn make_context() -> (Rc<SourceArena>, Rc<ErrorHandler>, Lexer) {
    let src = Rc::new(SourceArena::new());
    let eh = Rc::new(ErrorHandler::new(&src));
    let lx = Lexer::new(Reader::from_arena(&src), &eh);
    (src, eh, lx)
}

#[test]
fn lex_single_tokens() {
    let (src, mut eh, mut lx) = make_context();

    // actual test
    for tt in TokenType::iter() {
        src.intern(tokens_2_str(tt).to_string());
        src.intern(" ".to_string());
        assert_eq!(lx.next().unwrap().token_type, tt);
        assert_eq!(lx.next().unwrap().token_type, TokenType::Ignore);
    }
}

#[test]
fn lex_token_pairs() {
    let (src, mut eh, mut lx) = make_context();

    for (tt1, tt2) in iproduct!(TokenType::iter(), TokenType::iter()) {
        src.intern(tokens_2_str(tt1).to_string());
        src.intern(" ".to_string());
        src.intern(tokens_2_str(tt2).to_string());
        src.intern(" ".to_string());
        assert_eq!(lx.next().unwrap().token_type, tt1);
        assert_eq!(lx.next().unwrap().token_type, TokenType::Ignore);
        assert_eq!(lx.next().unwrap().token_type, tt2);
        assert_eq!(lx.next().unwrap().token_type, TokenType::Ignore);
    }
}

#[test]
fn lex_arbitrary_text() {
    let (src, mut eh, mut lx) = make_context();
    for s in two_char_strings!() {
        src.intern(s);
    }
    // just asserting that it lexes all the way through
    for _ in lx {}
}

#[test]
fn lex_int_literals() {
    let (src, mut eh, mut lx) = make_context();
    src.intern(format!("1234"));
    let t = lx.next().unwrap();
    assert_eq!(t.token_type, TokenType::IntLit);
    assert_eq!(t.lexeme.to_string(), format!("1234"));
}
