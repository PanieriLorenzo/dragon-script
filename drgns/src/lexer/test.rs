use strum::IntoEnumIterator;

use crate::arena::{intern, Reader};

use super::{test_utils::tokens_2_str, Lexer, TokenType};

fn make_lexer() -> Lexer {
    Lexer::new(Reader::new())
}

#[test]
fn lex_single_tokens() {
    let mut lx = make_lexer();
    for tt in TokenType::iter() {
        intern(tokens_2_str(tt).to_string());
        assert_eq!(lx.next().unwrap().token_type, tt);
    }
}
