#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use logos::Logos;
use std::fs::read_to_string;
mod misc {
    use indexmap::IndexSet;
    use rustc_hash::FxBuildHasher;
    use misc_utils::{KeyedVec, Key};
    use std::{
        ops::{Index, IndexMut},
        hash::Hash,
    };
    pub struct IndexedItemStore<K: Hash + PartialEq + Eq + Key, V> {
        all: KeyedVec<K, V>,
        roots: IndexSet<K, FxBuildHasher>,
    }
    #[automatically_derived]
    impl<K: ::core::fmt::Debug + Hash + PartialEq + Eq + Key, V: ::core::fmt::Debug>
        ::core::fmt::Debug for IndexedItemStore<K, V>
    {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "IndexedItemStore",
                "all",
                &self.all,
                "roots",
                &&self.roots,
            )
        }
    }
    impl<K: Hash + PartialEq + Eq + Key, V> IndexedItemStore<K, V> {
        pub fn insert(&mut self, val: V) -> K {
            self.all.insert(val)
        }
        pub fn add_root(&mut self, id: K) {
            self.roots.insert(id);
        }
        pub fn remove_root(&mut self, id: K) {
            self.roots.shift_remove(&id);
        }
        /// Iterates through the roots in order
        pub fn iter_roots(&self) -> impl Iterator<Item = &K> {
            self.roots.iter()
        }
    }
    impl<K: Hash + PartialEq + Eq + Key, V> Index<K> for IndexedItemStore<K, V> {
        type Output = V;
        fn index(&self, id: K) -> &V {
            &self.all[id]
        }
    }
    impl<K: Hash + PartialEq + Eq + Key, V> IndexMut<K> for IndexedItemStore<K, V> {
        fn index_mut(&mut self, id: K) -> &mut V {
            &mut self.all[id]
        }
    }
}
mod lexer {
    use parser_helper::Token as TokenTrait;
    use std::{
        fmt::{Display, Formatter, Result as FmtResult},
        error::Error,
    };
    use logos::{Lexer, Logos};
    pub use StartOrEnd::*;
    # [logos (skip "[ \t\r\n]")]
    # [logos (error = LexerError)]
    pub enum Token<'a> {
        #[regex("[^\"';#0-9\\\\()\\[\\]{} \t\r\n][^\"';\\\\()\\[\\]{} \t\r\n]*")]
        Ident(&'a str),
        #[regex(
            "[^\"';#0-9\\\\()\\[\\]{} \t\r\n][^\"';\\\\()\\[\\]{} \t\r\n]*/",
            parse_path
        )]
        Path(Vec<&'a str>),
        #[regex("[0-9][0-9_]*", parse_num)]
        #[regex("-[0-9_]+", parse_num_neg)]
        Number(i64),
        #[regex("[0-9]+\\.[0-9]+", parse_float)]
        #[regex("-[0-9]+\\.[0-9]+", parse_float_neg)]
        #[regex("[0-9]+\\.", invalid_float)]
        Float(f64),
        #[regex("#[a-zA-Z]+")]
        HashLiteral(&'a str),
        # [token ("\\space" , | _ |' ')]
        # [token ("\\newline" , | _ |'\n')]
        # [token ("\\tab" , | _ |'\n')]
        #[token("\\", parse_char)]
        Char(char),
        # [token ("(" , | _ | Start)]
        # [token (")" , | _ | End)]
        Paren(StartOrEnd),
        # [token ("[" , | _ | Start)]
        # [token ("]" , | _ | End)]
        Square(StartOrEnd),
        # [token ("{" , | _ | Start)]
        # [token ("}" , | _ | End)]
        Curly(StartOrEnd),
        #[token("'")]
        Quote,
        /// TODO: implement a proper string parser
        # [regex ("\"[^\"]*\"" , | l | l . slice () . to_string ())]
        String(String),
        #[regex(";[^\n]*")]
        Comment(&'a str),
        EOF,
    }
    #[automatically_derived]
    impl<'a> ::core::fmt::Debug for Token<'a> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                Token::Ident(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Ident", &__self_0)
                }
                Token::Path(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Path", &__self_0)
                }
                Token::Number(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Number", &__self_0)
                }
                Token::Float(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Float", &__self_0)
                }
                Token::HashLiteral(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "HashLiteral", &__self_0)
                }
                Token::Char(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Char", &__self_0)
                }
                Token::Paren(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Paren", &__self_0)
                }
                Token::Square(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Square", &__self_0)
                }
                Token::Curly(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Curly", &__self_0)
                }
                Token::Quote => ::core::fmt::Formatter::write_str(f, "Quote"),
                Token::String(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "String", &__self_0)
                }
                Token::Comment(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Comment", &__self_0)
                }
                Token::EOF => ::core::fmt::Formatter::write_str(f, "EOF"),
            }
        }
    }
    impl<'s> ::logos::Logos<'s> for Token<'s> {
        type Error = LexerError;
        type Extras = ();
        type Source = str;
        fn lex(lex: &mut ::logos::Lexer<'s, Self>) {
            use ::logos::internal::{LexerInternal, CallbackResult};
            type Lexer<'s> = ::logos::Lexer<'s, Token<'s>>;
            fn _end<'s>(lex: &mut Lexer<'s>) {
                lex.end()
            }
            fn _error<'s>(lex: &mut Lexer<'s>) {
                lex.bump_unchecked(1);
                lex.error();
            }
            static COMPACT_TABLE_0: [u8; 256] = [
                1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1,
            ];
            #[inline]
            fn goto3_ctx3_x<'s>(lex: &mut Lexer<'s>) {
                let token = Token::Ident(lex.slice());
                lex.set(Ok(token));
            }
            #[inline]
            fn goto13_ctx13_x<'s>(lex: &mut Lexer<'s>) {
                parse_path(lex).construct(Token::Path, lex);
            }
            #[inline]
            fn pattern0(byte: u8) -> bool {
                COMPACT_TABLE_0[byte as usize] & 1 > 0
            }
            #[inline]
            fn goto69_ctx13_x<'s>(lex: &mut Lexer<'s>) {
                let byte = match lex.read::<u8>() {
                    Some(byte) => byte,
                    None => return goto13_ctx13_x(lex),
                };
                match byte {
                    b'/' => {
                        lex.bump_unchecked(1usize);
                        goto69_ctx13_x(lex)
                    }
                    byte if pattern0(byte) => {
                        lex.bump_unchecked(1usize);
                        goto67_ctx3_x(lex)
                    }
                    _ => goto13_ctx13_x(lex),
                }
            }
            #[inline]
            fn goto67_ctx3_x<'s>(lex: &mut Lexer<'s>) {
                let byte = match lex.read::<u8>() {
                    Some(byte) => byte,
                    None => return goto3_ctx3_x(lex),
                };
                match byte {
                    b'/' => {
                        lex.bump_unchecked(1usize);
                        goto69_ctx13_x(lex)
                    }
                    byte if pattern0(byte) => {
                        lex.bump_unchecked(1usize);
                        goto67_ctx3_x(lex)
                    }
                    _ => goto3_ctx3_x(lex),
                }
            }
            #[inline]
            fn goto74_at1<'s>(lex: &mut Lexer<'s>) {
                match lex.read_at::<&[u8; 3usize]>(1usize) {
                    Some([144u8..=191u8, 128u8..=191u8, 128u8..=191u8]) => {
                        lex.bump_unchecked(4usize);
                        goto67_ctx3_x(lex)
                    }
                    _ => _error(lex),
                }
            }
            #[inline]
            fn goto71_at1<'s>(lex: &mut Lexer<'s>) {
                match lex.read_at::<&[u8; 2usize]>(1usize) {
                    Some([160u8..=191u8, 128u8..=191u8]) => {
                        lex.bump_unchecked(3usize);
                        goto67_ctx3_x(lex)
                    }
                    _ => _error(lex),
                }
            }
            #[inline]
            fn goto54_x<'s>(lex: &mut Lexer<'s>) {
                #[inline]
                fn callback<'s>(
                    _: &mut Lexer<'s>,
                ) -> impl CallbackResult<'s, StartOrEnd, Token<'s>> {
                    End
                }
                callback(lex).construct(Token::Paren, lex);
            }
            #[inline]
            fn goto1_x<'s>(lex: &mut Lexer<'s>) {
                lex.trivia();
                Token::lex(lex);
            }
            #[inline]
            fn goto64_ctx64_x<'s>(lex: &mut Lexer<'s>) {
                let token = Token::Comment(lex.slice());
                lex.set(Ok(token));
            }
            #[inline]
            fn pattern1(byte: u8) -> bool {
                match byte {
                    0u8..=9u8 | 11u8..=255u8 => true,
                    _ => false,
                }
            }
            #[inline]
            fn goto65_ctx64_x<'s>(lex: &mut Lexer<'s>) {
                while let Some(arr) = lex.read::<&[u8; 16]>() {
                    if pattern1(arr[0]) {
                        if pattern1(arr[1]) {
                            if pattern1(arr[2]) {
                                if pattern1(arr[3]) {
                                    if pattern1(arr[4]) {
                                        if pattern1(arr[5]) {
                                            if pattern1(arr[6]) {
                                                if pattern1(arr[7]) {
                                                    if pattern1(arr[8]) {
                                                        if pattern1(arr[9]) {
                                                            if pattern1(arr[10]) {
                                                                if pattern1(arr[11]) {
                                                                    if pattern1(arr[12]) {
                                                                        if pattern1(arr[13]) {
                                                                            if pattern1(arr[14]) {
                                                                                if pattern1(arr[15])
                                                                                {
                                                                                    lex . bump_unchecked (16) ;
                                                                                    continue;
                                                                                }
                                                                                lex.bump_unchecked(
                                                                                    15,
                                                                                );
                                                                                return goto64_ctx64_x (lex) ;
                                                                            }
                                                                            lex.bump_unchecked(14);
                                                                            return goto64_ctx64_x(
                                                                                lex,
                                                                            );
                                                                        }
                                                                        lex.bump_unchecked(13);
                                                                        return goto64_ctx64_x(lex);
                                                                    }
                                                                    lex.bump_unchecked(12);
                                                                    return goto64_ctx64_x(lex);
                                                                }
                                                                lex.bump_unchecked(11);
                                                                return goto64_ctx64_x(lex);
                                                            }
                                                            lex.bump_unchecked(10);
                                                            return goto64_ctx64_x(lex);
                                                        }
                                                        lex.bump_unchecked(9);
                                                        return goto64_ctx64_x(lex);
                                                    }
                                                    lex.bump_unchecked(8);
                                                    return goto64_ctx64_x(lex);
                                                }
                                                lex.bump_unchecked(7);
                                                return goto64_ctx64_x(lex);
                                            }
                                            lex.bump_unchecked(6);
                                            return goto64_ctx64_x(lex);
                                        }
                                        lex.bump_unchecked(5);
                                        return goto64_ctx64_x(lex);
                                    }
                                    lex.bump_unchecked(4);
                                    return goto64_ctx64_x(lex);
                                }
                                lex.bump_unchecked(3);
                                return goto64_ctx64_x(lex);
                            }
                            lex.bump_unchecked(2);
                            return goto64_ctx64_x(lex);
                        }
                        lex.bump_unchecked(1);
                        return goto64_ctx64_x(lex);
                    }
                    return goto64_ctx64_x(lex);
                }
                while lex.test(pattern1) {
                    lex.bump_unchecked(1);
                }
                goto64_ctx64_x(lex);
            }
            #[inline]
            fn goto45_ctx45_x<'s>(lex: &mut Lexer<'s>) {
                let token = Token::HashLiteral(lex.slice());
                lex.set(Ok(token));
            }
            #[inline]
            fn pattern2(byte: u8) -> bool {
                match byte {
                    b'A'..=b'Z' | b'a'..=b'z' => true,
                    _ => false,
                }
            }
            #[inline]
            fn goto46_ctx45_x<'s>(lex: &mut Lexer<'s>) {
                while let Some(arr) = lex.read::<&[u8; 16]>() {
                    if pattern2(arr[0]) {
                        if pattern2(arr[1]) {
                            if pattern2(arr[2]) {
                                if pattern2(arr[3]) {
                                    if pattern2(arr[4]) {
                                        if pattern2(arr[5]) {
                                            if pattern2(arr[6]) {
                                                if pattern2(arr[7]) {
                                                    if pattern2(arr[8]) {
                                                        if pattern2(arr[9]) {
                                                            if pattern2(arr[10]) {
                                                                if pattern2(arr[11]) {
                                                                    if pattern2(arr[12]) {
                                                                        if pattern2(arr[13]) {
                                                                            if pattern2(arr[14]) {
                                                                                if pattern2(arr[15])
                                                                                {
                                                                                    lex . bump_unchecked (16) ;
                                                                                    continue;
                                                                                }
                                                                                lex.bump_unchecked(
                                                                                    15,
                                                                                );
                                                                                return goto45_ctx45_x (lex) ;
                                                                            }
                                                                            lex.bump_unchecked(14);
                                                                            return goto45_ctx45_x(
                                                                                lex,
                                                                            );
                                                                        }
                                                                        lex.bump_unchecked(13);
                                                                        return goto45_ctx45_x(lex);
                                                                    }
                                                                    lex.bump_unchecked(12);
                                                                    return goto45_ctx45_x(lex);
                                                                }
                                                                lex.bump_unchecked(11);
                                                                return goto45_ctx45_x(lex);
                                                            }
                                                            lex.bump_unchecked(10);
                                                            return goto45_ctx45_x(lex);
                                                        }
                                                        lex.bump_unchecked(9);
                                                        return goto45_ctx45_x(lex);
                                                    }
                                                    lex.bump_unchecked(8);
                                                    return goto45_ctx45_x(lex);
                                                }
                                                lex.bump_unchecked(7);
                                                return goto45_ctx45_x(lex);
                                            }
                                            lex.bump_unchecked(6);
                                            return goto45_ctx45_x(lex);
                                        }
                                        lex.bump_unchecked(5);
                                        return goto45_ctx45_x(lex);
                                    }
                                    lex.bump_unchecked(4);
                                    return goto45_ctx45_x(lex);
                                }
                                lex.bump_unchecked(3);
                                return goto45_ctx45_x(lex);
                            }
                            lex.bump_unchecked(2);
                            return goto45_ctx45_x(lex);
                        }
                        lex.bump_unchecked(1);
                        return goto45_ctx45_x(lex);
                    }
                    return goto45_ctx45_x(lex);
                }
                while lex.test(pattern2) {
                    lex.bump_unchecked(1);
                }
                goto45_ctx45_x(lex);
            }
            #[inline]
            fn goto47_at1<'s>(lex: &mut Lexer<'s>) {
                let byte = match lex.read_at::<u8>(1usize) {
                    Some(byte) => byte,
                    None => return _error(lex),
                };
                match byte {
                    byte if pattern2(byte) => {
                        lex.bump_unchecked(2usize);
                        goto46_ctx45_x(lex)
                    }
                    _ => _error(lex),
                }
            }
            #[inline]
            fn goto60_x<'s>(lex: &mut Lexer<'s>) {
                #[inline]
                fn callback<'s>(l: &mut Lexer<'s>) -> impl CallbackResult<'s, String, Token<'s>> {
                    l.slice().to_string()
                }
                callback(lex).construct(Token::String, lex);
            }
            #[inline]
            fn goto61_x<'s>(lex: &mut Lexer<'s>) {
                match lex.read::<&[u8; 1usize]>() {
                    Some(b"\"") => {
                        lex.bump_unchecked(1usize);
                        goto60_x(lex)
                    }
                    _ => lex.error(),
                }
            }
            #[inline]
            fn goto60_ctx61_x<'s>(lex: &mut Lexer<'s>) {
                #[inline]
                fn callback<'s>(l: &mut Lexer<'s>) -> impl CallbackResult<'s, String, Token<'s>> {
                    l.slice().to_string()
                }
                callback(lex).construct(Token::String, lex);
            }
            #[inline]
            fn goto61_ctx61_x<'s>(lex: &mut Lexer<'s>) {
                match lex.read::<&[u8; 1usize]>() {
                    Some(b"\"") => {
                        lex.bump_unchecked(1usize);
                        goto60_ctx61_x(lex)
                    }
                    _ => goto61_x(lex),
                }
            }
            #[inline]
            fn pattern3(byte: u8) -> bool {
                match byte {
                    0u8..=b'!' | b'#'..=255u8 => true,
                    _ => false,
                }
            }
            #[inline]
            fn goto62_ctx61_x<'s>(lex: &mut Lexer<'s>) {
                while let Some(arr) = lex.read::<&[u8; 16]>() {
                    if pattern3(arr[0]) {
                        if pattern3(arr[1]) {
                            if pattern3(arr[2]) {
                                if pattern3(arr[3]) {
                                    if pattern3(arr[4]) {
                                        if pattern3(arr[5]) {
                                            if pattern3(arr[6]) {
                                                if pattern3(arr[7]) {
                                                    if pattern3(arr[8]) {
                                                        if pattern3(arr[9]) {
                                                            if pattern3(arr[10]) {
                                                                if pattern3(arr[11]) {
                                                                    if pattern3(arr[12]) {
                                                                        if pattern3(arr[13]) {
                                                                            if pattern3(arr[14]) {
                                                                                if pattern3(arr[15])
                                                                                {
                                                                                    lex . bump_unchecked (16) ;
                                                                                    continue;
                                                                                }
                                                                                lex.bump_unchecked(
                                                                                    15,
                                                                                );
                                                                                return goto61_ctx61_x (lex) ;
                                                                            }
                                                                            lex.bump_unchecked(14);
                                                                            return goto61_ctx61_x(
                                                                                lex,
                                                                            );
                                                                        }
                                                                        lex.bump_unchecked(13);
                                                                        return goto61_ctx61_x(lex);
                                                                    }
                                                                    lex.bump_unchecked(12);
                                                                    return goto61_ctx61_x(lex);
                                                                }
                                                                lex.bump_unchecked(11);
                                                                return goto61_ctx61_x(lex);
                                                            }
                                                            lex.bump_unchecked(10);
                                                            return goto61_ctx61_x(lex);
                                                        }
                                                        lex.bump_unchecked(9);
                                                        return goto61_ctx61_x(lex);
                                                    }
                                                    lex.bump_unchecked(8);
                                                    return goto61_ctx61_x(lex);
                                                }
                                                lex.bump_unchecked(7);
                                                return goto61_ctx61_x(lex);
                                            }
                                            lex.bump_unchecked(6);
                                            return goto61_ctx61_x(lex);
                                        }
                                        lex.bump_unchecked(5);
                                        return goto61_ctx61_x(lex);
                                    }
                                    lex.bump_unchecked(4);
                                    return goto61_ctx61_x(lex);
                                }
                                lex.bump_unchecked(3);
                                return goto61_ctx61_x(lex);
                            }
                            lex.bump_unchecked(2);
                            return goto61_ctx61_x(lex);
                        }
                        lex.bump_unchecked(1);
                        return goto61_ctx61_x(lex);
                    }
                    return goto61_ctx61_x(lex);
                }
                while lex.test(pattern3) {
                    lex.bump_unchecked(1);
                }
                goto61_ctx61_x(lex);
            }
            #[inline]
            fn goto73_at1<'s>(lex: &mut Lexer<'s>) {
                match lex.read_at::<&[u8; 2usize]>(1usize) {
                    Some([128u8..=159u8, 128u8..=191u8]) => {
                        lex.bump_unchecked(3usize);
                        goto67_ctx3_x(lex)
                    }
                    _ => _error(lex),
                }
            }
            #[inline]
            fn goto56_x<'s>(lex: &mut Lexer<'s>) {
                #[inline]
                fn callback<'s>(
                    _: &mut Lexer<'s>,
                ) -> impl CallbackResult<'s, StartOrEnd, Token<'s>> {
                    End
                }
                callback(lex).construct(Token::Square, lex);
            }
            #[inline]
            fn goto75_at1<'s>(lex: &mut Lexer<'s>) {
                match lex.read_at::<&[u8; 3usize]>(1usize) {
                    Some([128u8..=191u8, 128u8..=191u8, 128u8..=191u8]) => {
                        lex.bump_unchecked(4usize);
                        goto67_ctx3_x(lex)
                    }
                    _ => _error(lex),
                }
            }
            #[inline]
            fn goto58_x<'s>(lex: &mut Lexer<'s>) {
                #[inline]
                fn callback<'s>(
                    _: &mut Lexer<'s>,
                ) -> impl CallbackResult<'s, StartOrEnd, Token<'s>> {
                    End
                }
                callback(lex).construct(Token::Curly, lex);
            }
            #[inline]
            fn goto53_x<'s>(lex: &mut Lexer<'s>) {
                #[inline]
                fn callback<'s>(
                    _: &mut Lexer<'s>,
                ) -> impl CallbackResult<'s, StartOrEnd, Token<'s>> {
                    Start
                }
                callback(lex).construct(Token::Paren, lex);
            }
            #[inline]
            fn goto52_ctx52_x<'s>(lex: &mut Lexer<'s>) {
                parse_char(lex).construct(Token::Char, lex);
            }
            #[inline]
            fn goto52_x<'s>(lex: &mut Lexer<'s>) {
                parse_char(lex).construct(Token::Char, lex);
            }
            #[inline]
            fn goto50_ctx52_x<'s>(lex: &mut Lexer<'s>) {
                #[inline]
                fn callback<'s>(_: &mut Lexer<'s>) -> impl CallbackResult<'s, char, Token<'s>> {
                    '\n'
                }
                callback(lex).construct(Token::Char, lex);
            }
            #[inline]
            fn goto96_at1_ctx52_x<'s>(lex: &mut Lexer<'s>) {
                match lex.read_at::<&[u8; 6usize]>(1usize) {
                    Some(b"ewline") => {
                        lex.bump_unchecked(7usize);
                        goto50_ctx52_x(lex)
                    }
                    _ => goto52_x(lex),
                }
            }
            #[inline]
            fn goto49_ctx52_x<'s>(lex: &mut Lexer<'s>) {
                #[inline]
                fn callback<'s>(_: &mut Lexer<'s>) -> impl CallbackResult<'s, char, Token<'s>> {
                    ' '
                }
                callback(lex).construct(Token::Char, lex);
            }
            #[inline]
            fn goto95_at1_ctx52_x<'s>(lex: &mut Lexer<'s>) {
                match lex.read_at::<&[u8; 4usize]>(1usize) {
                    Some(b"pace") => {
                        lex.bump_unchecked(5usize);
                        goto49_ctx52_x(lex)
                    }
                    _ => goto52_x(lex),
                }
            }
            #[inline]
            fn goto51_ctx52_x<'s>(lex: &mut Lexer<'s>) {
                #[inline]
                fn callback<'s>(_: &mut Lexer<'s>) -> impl CallbackResult<'s, char, Token<'s>> {
                    '\n'
                }
                callback(lex).construct(Token::Char, lex);
            }
            #[inline]
            fn goto99_at1_ctx52_x<'s>(lex: &mut Lexer<'s>) {
                match lex.read_at::<&[u8; 2usize]>(1usize) {
                    Some(b"ab") => {
                        lex.bump_unchecked(3usize);
                        goto51_ctx52_x(lex)
                    }
                    _ => goto52_x(lex),
                }
            }
            #[inline]
            fn goto100_ctx52_x<'s>(lex: &mut Lexer<'s>) {
                enum Jump {
                    __,
                    J96,
                    J95,
                    J99,
                }
                const LUT: [Jump; 256] = {
                    use Jump::*;
                    [
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, J96, __, __,
                        __, __, J95, J99, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __,
                    ]
                };
                let byte = match lex.read::<u8>() {
                    Some(byte) => byte,
                    None => return goto52_ctx52_x(lex),
                };
                match LUT[byte as usize] {
                    Jump::J96 => goto96_at1_ctx52_x(lex),
                    Jump::J95 => goto95_at1_ctx52_x(lex),
                    Jump::J99 => goto99_at1_ctx52_x(lex),
                    Jump::__ => goto52_ctx52_x(lex),
                }
            }
            #[inline]
            fn goto55_x<'s>(lex: &mut Lexer<'s>) {
                #[inline]
                fn callback<'s>(
                    _: &mut Lexer<'s>,
                ) -> impl CallbackResult<'s, StartOrEnd, Token<'s>> {
                    Start
                }
                callback(lex).construct(Token::Square, lex);
            }
            #[inline]
            fn goto70_at1<'s>(lex: &mut Lexer<'s>) {
                match lex.read_at::<&[u8; 1usize]>(1usize) {
                    Some([128u8..=191u8]) => {
                        lex.bump_unchecked(2usize);
                        goto67_ctx3_x(lex)
                    }
                    _ => _error(lex),
                }
            }
            #[inline]
            fn goto27_ctx27_x<'s>(lex: &mut Lexer<'s>) {
                parse_num_neg(lex).construct(Token::Number, lex);
            }
            #[inline]
            fn goto36_ctx36_x<'s>(lex: &mut Lexer<'s>) {
                parse_float_neg(lex).construct(Token::Float, lex);
            }
            #[inline]
            fn goto88_ctx36_x<'s>(lex: &mut Lexer<'s>) {
                enum Jump {
                    __,
                    J69,
                    J88,
                    J67,
                }
                const LUT: [Jump; 256] = {
                    use Jump::*;
                    [
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, __, __, J67, J67, __, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, __, J67, __, J67, J67, J67, J67, __, __, __, J67, J67, J67, J67,
                        J67, J69, J88, J88, J88, J88, J88, J88, J88, J88, J88, J88, J67, __, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        __, __, __, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, __, J67, __, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                    ]
                };
                let byte = match lex.read::<u8>() {
                    Some(byte) => byte,
                    None => return goto36_ctx36_x(lex),
                };
                match LUT[byte as usize] {
                    Jump::J69 => {
                        lex.bump_unchecked(1usize);
                        goto69_ctx13_x(lex)
                    }
                    Jump::J88 => {
                        lex.bump_unchecked(1usize);
                        goto88_ctx36_x(lex)
                    }
                    Jump::J67 => {
                        lex.bump_unchecked(1usize);
                        goto67_ctx3_x(lex)
                    }
                    Jump::__ => goto36_ctx36_x(lex),
                }
            }
            #[inline]
            fn goto87_ctx27_x<'s>(lex: &mut Lexer<'s>) {
                match lex.read::<&[u8; 1usize]>() {
                    Some([b'0'..=b'9']) => {
                        lex.bump_unchecked(1usize);
                        goto88_ctx36_x(lex)
                    }
                    _ => goto67_ctx3_x(lex),
                }
            }
            #[inline]
            fn goto78_ctx27_x<'s>(lex: &mut Lexer<'s>) {
                enum Jump {
                    __,
                    J69,
                    J67,
                    J78,
                }
                const LUT: [Jump; 256] = {
                    use Jump::*;
                    [
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, __, __, J67, J67, __, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, __, J67, __, J67, J67, J67, J67, __, __, __, J67, J67, J67, J67,
                        J67, J69, J78, J78, J78, J78, J78, J78, J78, J78, J78, J78, J67, __, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        __, __, __, J67, J78, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, __, J67, __, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                    ]
                };
                let byte = match lex.read::<u8>() {
                    Some(byte) => byte,
                    None => return goto27_ctx27_x(lex),
                };
                match LUT[byte as usize] {
                    Jump::J69 => {
                        lex.bump_unchecked(1usize);
                        goto69_ctx13_x(lex)
                    }
                    Jump::J67 => {
                        lex.bump_unchecked(1usize);
                        goto67_ctx3_x(lex)
                    }
                    Jump::J78 => {
                        lex.bump_unchecked(1usize);
                        goto78_ctx27_x(lex)
                    }
                    Jump::__ => goto27_ctx27_x(lex),
                }
            }
            #[inline]
            fn goto84_ctx27_x<'s>(lex: &mut Lexer<'s>) {
                enum Jump {
                    __,
                    J69,
                    J84,
                    J87,
                    J78,
                    J67,
                }
                const LUT: [Jump; 256] = {
                    use Jump::*;
                    [
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, __, __, J67, J67, __, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, __, J67, __, J67, J67, J67, J67, __, __, __, J67, J67, J67, J67,
                        J87, J69, J84, J84, J84, J84, J84, J84, J84, J84, J84, J84, J67, __, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        __, __, __, J67, J78, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, __, J67, __, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                    ]
                };
                let byte = match lex.read::<u8>() {
                    Some(byte) => byte,
                    None => return goto27_ctx27_x(lex),
                };
                match LUT[byte as usize] {
                    Jump::J69 => {
                        lex.bump_unchecked(1usize);
                        goto69_ctx13_x(lex)
                    }
                    Jump::J84 => {
                        lex.bump_unchecked(1usize);
                        goto84_ctx27_x(lex)
                    }
                    Jump::J87 => {
                        lex.bump_unchecked(1usize);
                        goto87_ctx27_x(lex)
                    }
                    Jump::J78 => {
                        lex.bump_unchecked(1usize);
                        goto78_ctx27_x(lex)
                    }
                    Jump::J67 => {
                        lex.bump_unchecked(1usize);
                        goto67_ctx3_x(lex)
                    }
                    Jump::__ => goto27_ctx27_x(lex),
                }
            }
            #[inline]
            fn goto83_ctx3_x<'s>(lex: &mut Lexer<'s>) {
                enum Jump {
                    __,
                    J69,
                    J84,
                    J78,
                    J67,
                }
                const LUT: [Jump; 256] = {
                    use Jump::*;
                    [
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, __, __, J67, J67, __, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, __, J67, __, J67, J67, J67, J67, __, __, __, J67, J67, J67, J67,
                        J67, J69, J84, J84, J84, J84, J84, J84, J84, J84, J84, J84, J67, __, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        __, __, __, J67, J78, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, __, J67, __, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                    ]
                };
                let byte = match lex.read::<u8>() {
                    Some(byte) => byte,
                    None => return goto3_ctx3_x(lex),
                };
                match LUT[byte as usize] {
                    Jump::J69 => {
                        lex.bump_unchecked(1usize);
                        goto69_ctx13_x(lex)
                    }
                    Jump::J84 => {
                        lex.bump_unchecked(1usize);
                        goto84_ctx27_x(lex)
                    }
                    Jump::J78 => {
                        lex.bump_unchecked(1usize);
                        goto78_ctx27_x(lex)
                    }
                    Jump::J67 => {
                        lex.bump_unchecked(1usize);
                        goto67_ctx3_x(lex)
                    }
                    Jump::__ => goto3_ctx3_x(lex),
                }
            }
            #[inline]
            fn goto76_at1<'s>(lex: &mut Lexer<'s>) {
                match lex.read_at::<&[u8; 3usize]>(1usize) {
                    Some([128u8..=143u8, 128u8..=191u8, 128u8..=191u8]) => {
                        lex.bump_unchecked(4usize);
                        goto67_ctx3_x(lex)
                    }
                    _ => _error(lex),
                }
            }
            #[inline]
            fn goto24_ctx24_x<'s>(lex: &mut Lexer<'s>) {
                parse_num(lex).construct(Token::Number, lex);
            }
            #[inline]
            fn pattern4(byte: u8) -> bool {
                match byte {
                    b'0'..=b'9' | b'_' => true,
                    _ => false,
                }
            }
            #[inline]
            fn goto25_ctx24_x<'s>(lex: &mut Lexer<'s>) {
                while let Some(arr) = lex.read::<&[u8; 16]>() {
                    if pattern4(arr[0]) {
                        if pattern4(arr[1]) {
                            if pattern4(arr[2]) {
                                if pattern4(arr[3]) {
                                    if pattern4(arr[4]) {
                                        if pattern4(arr[5]) {
                                            if pattern4(arr[6]) {
                                                if pattern4(arr[7]) {
                                                    if pattern4(arr[8]) {
                                                        if pattern4(arr[9]) {
                                                            if pattern4(arr[10]) {
                                                                if pattern4(arr[11]) {
                                                                    if pattern4(arr[12]) {
                                                                        if pattern4(arr[13]) {
                                                                            if pattern4(arr[14]) {
                                                                                if pattern4(arr[15])
                                                                                {
                                                                                    lex . bump_unchecked (16) ;
                                                                                    continue;
                                                                                }
                                                                                lex.bump_unchecked(
                                                                                    15,
                                                                                );
                                                                                return goto24_ctx24_x (lex) ;
                                                                            }
                                                                            lex.bump_unchecked(14);
                                                                            return goto24_ctx24_x(
                                                                                lex,
                                                                            );
                                                                        }
                                                                        lex.bump_unchecked(13);
                                                                        return goto24_ctx24_x(lex);
                                                                    }
                                                                    lex.bump_unchecked(12);
                                                                    return goto24_ctx24_x(lex);
                                                                }
                                                                lex.bump_unchecked(11);
                                                                return goto24_ctx24_x(lex);
                                                            }
                                                            lex.bump_unchecked(10);
                                                            return goto24_ctx24_x(lex);
                                                        }
                                                        lex.bump_unchecked(9);
                                                        return goto24_ctx24_x(lex);
                                                    }
                                                    lex.bump_unchecked(8);
                                                    return goto24_ctx24_x(lex);
                                                }
                                                lex.bump_unchecked(7);
                                                return goto24_ctx24_x(lex);
                                            }
                                            lex.bump_unchecked(6);
                                            return goto24_ctx24_x(lex);
                                        }
                                        lex.bump_unchecked(5);
                                        return goto24_ctx24_x(lex);
                                    }
                                    lex.bump_unchecked(4);
                                    return goto24_ctx24_x(lex);
                                }
                                lex.bump_unchecked(3);
                                return goto24_ctx24_x(lex);
                            }
                            lex.bump_unchecked(2);
                            return goto24_ctx24_x(lex);
                        }
                        lex.bump_unchecked(1);
                        return goto24_ctx24_x(lex);
                    }
                    return goto24_ctx24_x(lex);
                }
                while lex.test(pattern4) {
                    lex.bump_unchecked(1);
                }
                goto24_ctx24_x(lex);
            }
            #[inline]
            fn goto41_ctx24_x<'s>(lex: &mut Lexer<'s>) {
                invalid_float(lex).construct(Token::Float, lex);
            }
            #[inline]
            fn goto31_ctx31_x<'s>(lex: &mut Lexer<'s>) {
                parse_float(lex).construct(Token::Float, lex);
            }
            #[inline]
            fn pattern5(byte: u8) -> bool {
                match byte {
                    b'0'..=b'9' => true,
                    _ => false,
                }
            }
            #[inline]
            fn goto32_ctx31_x<'s>(lex: &mut Lexer<'s>) {
                while let Some(arr) = lex.read::<&[u8; 16]>() {
                    if pattern5(arr[0]) {
                        if pattern5(arr[1]) {
                            if pattern5(arr[2]) {
                                if pattern5(arr[3]) {
                                    if pattern5(arr[4]) {
                                        if pattern5(arr[5]) {
                                            if pattern5(arr[6]) {
                                                if pattern5(arr[7]) {
                                                    if pattern5(arr[8]) {
                                                        if pattern5(arr[9]) {
                                                            if pattern5(arr[10]) {
                                                                if pattern5(arr[11]) {
                                                                    if pattern5(arr[12]) {
                                                                        if pattern5(arr[13]) {
                                                                            if pattern5(arr[14]) {
                                                                                if pattern5(arr[15])
                                                                                {
                                                                                    lex . bump_unchecked (16) ;
                                                                                    continue;
                                                                                }
                                                                                lex.bump_unchecked(
                                                                                    15,
                                                                                );
                                                                                return goto31_ctx31_x (lex) ;
                                                                            }
                                                                            lex.bump_unchecked(14);
                                                                            return goto31_ctx31_x(
                                                                                lex,
                                                                            );
                                                                        }
                                                                        lex.bump_unchecked(13);
                                                                        return goto31_ctx31_x(lex);
                                                                    }
                                                                    lex.bump_unchecked(12);
                                                                    return goto31_ctx31_x(lex);
                                                                }
                                                                lex.bump_unchecked(11);
                                                                return goto31_ctx31_x(lex);
                                                            }
                                                            lex.bump_unchecked(10);
                                                            return goto31_ctx31_x(lex);
                                                        }
                                                        lex.bump_unchecked(9);
                                                        return goto31_ctx31_x(lex);
                                                    }
                                                    lex.bump_unchecked(8);
                                                    return goto31_ctx31_x(lex);
                                                }
                                                lex.bump_unchecked(7);
                                                return goto31_ctx31_x(lex);
                                            }
                                            lex.bump_unchecked(6);
                                            return goto31_ctx31_x(lex);
                                        }
                                        lex.bump_unchecked(5);
                                        return goto31_ctx31_x(lex);
                                    }
                                    lex.bump_unchecked(4);
                                    return goto31_ctx31_x(lex);
                                }
                                lex.bump_unchecked(3);
                                return goto31_ctx31_x(lex);
                            }
                            lex.bump_unchecked(2);
                            return goto31_ctx31_x(lex);
                        }
                        lex.bump_unchecked(1);
                        return goto31_ctx31_x(lex);
                    }
                    return goto31_ctx31_x(lex);
                }
                while lex.test(pattern5) {
                    lex.bump_unchecked(1);
                }
                goto31_ctx31_x(lex);
            }
            #[inline]
            fn goto91_ctx24_x<'s>(lex: &mut Lexer<'s>) {
                match lex.read::<&[u8; 1usize]>() {
                    Some([b'0'..=b'9']) => {
                        lex.bump_unchecked(1usize);
                        goto32_ctx31_x(lex)
                    }
                    _ => goto41_ctx24_x(lex),
                }
            }
            #[inline]
            fn goto89_ctx24_x<'s>(lex: &mut Lexer<'s>) {
                enum Jump {
                    __,
                    J89,
                    J25,
                    J91,
                }
                const LUT: [Jump; 256] = {
                    use Jump::*;
                    [
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, J91, __, J89, J89, J89, J89, J89, J89, J89,
                        J89, J89, J89, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, J25, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __,
                    ]
                };
                let byte = match lex.read::<u8>() {
                    Some(byte) => byte,
                    None => return goto24_ctx24_x(lex),
                };
                match LUT[byte as usize] {
                    Jump::J89 => {
                        lex.bump_unchecked(1usize);
                        goto89_ctx24_x(lex)
                    }
                    Jump::J25 => {
                        lex.bump_unchecked(1usize);
                        goto25_ctx24_x(lex)
                    }
                    Jump::J91 => {
                        lex.bump_unchecked(1usize);
                        goto91_ctx24_x(lex)
                    }
                    Jump::__ => goto24_ctx24_x(lex),
                }
            }
            #[inline]
            fn goto57_x<'s>(lex: &mut Lexer<'s>) {
                #[inline]
                fn callback<'s>(
                    _: &mut Lexer<'s>,
                ) -> impl CallbackResult<'s, StartOrEnd, Token<'s>> {
                    Start
                }
                callback(lex).construct(Token::Curly, lex);
            }
            #[inline]
            fn goto59_x<'s>(lex: &mut Lexer<'s>) {
                lex.set(Ok(Token::Quote));
            }
            #[inline]
            fn goto72_at1<'s>(lex: &mut Lexer<'s>) {
                match lex.read_at::<&[u8; 2usize]>(1usize) {
                    Some([128u8..=191u8, 128u8..=191u8]) => {
                        lex.bump_unchecked(3usize);
                        goto67_ctx3_x(lex)
                    }
                    _ => _error(lex),
                }
            }
            #[inline]
            fn goto101<'s>(lex: &mut Lexer<'s>) {
                enum Jump {
                    __,
                    J74,
                    J71,
                    J54,
                    J1,
                    J65,
                    J67,
                    J47,
                    J62,
                    J73,
                    J56,
                    J75,
                    J58,
                    J53,
                    J100,
                    J55,
                    J70,
                    J83,
                    J76,
                    J89,
                    J57,
                    J59,
                    J72,
                }
                const LUT: [Jump; 256] = {
                    use Jump::*;
                    [
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J1, J1, J67, J67, J1, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J1, J67, J62, J47, J67, J67, J67, J59, J53, J54, J67, J67, J67,
                        J83, J67, J67, J89, J89, J89, J89, J89, J89, J89, J89, J89, J89, J67, J65,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J55, J100, J56, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67, J67,
                        J67, J67, J67, J57, J67, J58, J67, J67, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __,
                        J70, J70, J70, J70, J70, J70, J70, J70, J70, J70, J70, J70, J70, J70, J70,
                        J70, J70, J70, J70, J70, J70, J70, J70, J70, J70, J70, J70, J70, J70, J70,
                        J71, J72, J72, J72, J72, J72, J72, J72, J72, J72, J72, J72, J72, J73, J72,
                        J72, J74, J75, J75, J75, J76, __, __, __, __, __, __, __, __, __, __, __,
                    ]
                };
                let byte = match lex.read::<u8>() {
                    Some(byte) => byte,
                    None => return _end(lex),
                };
                match LUT[byte as usize] {
                    Jump::J74 => goto74_at1(lex),
                    Jump::J71 => goto71_at1(lex),
                    Jump::J54 => {
                        lex.bump_unchecked(1usize);
                        goto54_x(lex)
                    }
                    Jump::J1 => {
                        lex.bump_unchecked(1usize);
                        goto1_x(lex)
                    }
                    Jump::J65 => {
                        lex.bump_unchecked(1usize);
                        goto65_ctx64_x(lex)
                    }
                    Jump::J67 => {
                        lex.bump_unchecked(1usize);
                        goto67_ctx3_x(lex)
                    }
                    Jump::J47 => goto47_at1(lex),
                    Jump::J62 => {
                        lex.bump_unchecked(1usize);
                        goto62_ctx61_x(lex)
                    }
                    Jump::J73 => goto73_at1(lex),
                    Jump::J56 => {
                        lex.bump_unchecked(1usize);
                        goto56_x(lex)
                    }
                    Jump::J75 => goto75_at1(lex),
                    Jump::J58 => {
                        lex.bump_unchecked(1usize);
                        goto58_x(lex)
                    }
                    Jump::J53 => {
                        lex.bump_unchecked(1usize);
                        goto53_x(lex)
                    }
                    Jump::J100 => {
                        lex.bump_unchecked(1usize);
                        goto100_ctx52_x(lex)
                    }
                    Jump::J55 => {
                        lex.bump_unchecked(1usize);
                        goto55_x(lex)
                    }
                    Jump::J70 => goto70_at1(lex),
                    Jump::J83 => {
                        lex.bump_unchecked(1usize);
                        goto83_ctx3_x(lex)
                    }
                    Jump::J76 => goto76_at1(lex),
                    Jump::J89 => {
                        lex.bump_unchecked(1usize);
                        goto89_ctx24_x(lex)
                    }
                    Jump::J57 => {
                        lex.bump_unchecked(1usize);
                        goto57_x(lex)
                    }
                    Jump::J59 => {
                        lex.bump_unchecked(1usize);
                        goto59_x(lex)
                    }
                    Jump::J72 => goto72_at1(lex),
                    Jump::__ => _error(lex),
                }
            }
            goto101(lex)
        }
    }
    #[automatically_derived]
    impl<'a> ::core::marker::StructuralPartialEq for Token<'a> {}
    #[automatically_derived]
    impl<'a> ::core::cmp::PartialEq for Token<'a> {
        #[inline]
        fn eq(&self, other: &Token<'a>) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
                && match (self, other) {
                    (Token::Ident(__self_0), Token::Ident(__arg1_0)) => __self_0 == __arg1_0,
                    (Token::Path(__self_0), Token::Path(__arg1_0)) => __self_0 == __arg1_0,
                    (Token::Number(__self_0), Token::Number(__arg1_0)) => __self_0 == __arg1_0,
                    (Token::Float(__self_0), Token::Float(__arg1_0)) => __self_0 == __arg1_0,
                    (Token::HashLiteral(__self_0), Token::HashLiteral(__arg1_0)) => {
                        __self_0 == __arg1_0
                    }
                    (Token::Char(__self_0), Token::Char(__arg1_0)) => __self_0 == __arg1_0,
                    (Token::Paren(__self_0), Token::Paren(__arg1_0)) => __self_0 == __arg1_0,
                    (Token::Square(__self_0), Token::Square(__arg1_0)) => __self_0 == __arg1_0,
                    (Token::Curly(__self_0), Token::Curly(__arg1_0)) => __self_0 == __arg1_0,
                    (Token::String(__self_0), Token::String(__arg1_0)) => __self_0 == __arg1_0,
                    (Token::Comment(__self_0), Token::Comment(__arg1_0)) => __self_0 == __arg1_0,
                    _ => true,
                }
        }
    }
    #[automatically_derived]
    impl<'a> ::core::clone::Clone for Token<'a> {
        #[inline]
        fn clone(&self) -> Token<'a> {
            match self {
                Token::Ident(__self_0) => Token::Ident(::core::clone::Clone::clone(__self_0)),
                Token::Path(__self_0) => Token::Path(::core::clone::Clone::clone(__self_0)),
                Token::Number(__self_0) => Token::Number(::core::clone::Clone::clone(__self_0)),
                Token::Float(__self_0) => Token::Float(::core::clone::Clone::clone(__self_0)),
                Token::HashLiteral(__self_0) => {
                    Token::HashLiteral(::core::clone::Clone::clone(__self_0))
                }
                Token::Char(__self_0) => Token::Char(::core::clone::Clone::clone(__self_0)),
                Token::Paren(__self_0) => Token::Paren(::core::clone::Clone::clone(__self_0)),
                Token::Square(__self_0) => Token::Square(::core::clone::Clone::clone(__self_0)),
                Token::Curly(__self_0) => Token::Curly(::core::clone::Clone::clone(__self_0)),
                Token::Quote => Token::Quote,
                Token::String(__self_0) => Token::String(::core::clone::Clone::clone(__self_0)),
                Token::Comment(__self_0) => Token::Comment(::core::clone::Clone::clone(__self_0)),
                Token::EOF => Token::EOF,
            }
        }
    }
    impl<'a> TokenTrait for Token<'a> {
        fn eof() -> Self {
            Self::EOF
        }
    }
    pub enum StartOrEnd {
        Start,
        End,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for StartOrEnd {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    StartOrEnd::Start => "Start",
                    StartOrEnd::End => "End",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for StartOrEnd {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for StartOrEnd {
        #[inline]
        fn eq(&self, other: &StartOrEnd) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for StartOrEnd {
        #[inline]
        fn clone(&self) -> StartOrEnd {
            match self {
                StartOrEnd::Start => StartOrEnd::Start,
                StartOrEnd::End => StartOrEnd::End,
            }
        }
    }
    pub enum LexerError {
        EmptyPathSegment,
        IntegerOverflow,
        FloatOverflow,
        UnexpectedEof,
        InvalidFloat,
        InvalidToken,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for LexerError {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    LexerError::EmptyPathSegment => "EmptyPathSegment",
                    LexerError::IntegerOverflow => "IntegerOverflow",
                    LexerError::FloatOverflow => "FloatOverflow",
                    LexerError::UnexpectedEof => "UnexpectedEof",
                    LexerError::InvalidFloat => "InvalidFloat",
                    LexerError::InvalidToken => "InvalidToken",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for LexerError {
        #[inline]
        fn clone(&self) -> LexerError {
            match self {
                LexerError::EmptyPathSegment => LexerError::EmptyPathSegment,
                LexerError::IntegerOverflow => LexerError::IntegerOverflow,
                LexerError::FloatOverflow => LexerError::FloatOverflow,
                LexerError::UnexpectedEof => LexerError::UnexpectedEof,
                LexerError::InvalidFloat => LexerError::InvalidFloat,
                LexerError::InvalidToken => LexerError::InvalidToken,
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for LexerError {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for LexerError {
        #[inline]
        fn eq(&self, other: &LexerError) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    impl Default for LexerError {
        fn default() -> Self {
            LexerError::InvalidToken
        }
    }
    impl Error for LexerError {}
    impl Display for LexerError {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            use LexerError::*;
            match self {
                EmptyPathSegment => f.write_fmt(format_args!("Empty segment in path")),
                IntegerOverflow => f.write_fmt(format_args!("Integer Overflow")),
                FloatOverflow => f.write_fmt(format_args!("Float overflow")),
                UnexpectedEof => f.write_fmt(format_args!("Unexpected EOF")),
                InvalidFloat => f.write_fmt(format_args!("Invalid Float")),
                InvalidToken => f.write_fmt(format_args!("Invalid Token")),
            }
        }
    }
    fn parse_path<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Vec<&'a str>, LexerError> {
        let mut out = Vec::new();
        let len = lex.slice().len();
        out.push(&lex.slice()[..(len - 1)]);
        let mut err = false;
        let mut slice_start = 0;
        let mut count = 0;
        for c in lex.remainder().chars() {
            if slice_start == count {
                match c {
                    '/' => err = true,
                    ';'
                    | '\\'
                    | ' '
                    | '\t'
                    | '\r'
                    | '\n'
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '"'
                    | '\''
                    | '#'
                    | '0'..='9' => break,
                    _ => {}
                }
            } else {
                match c {
                    '/' => {
                        out.push(&lex.remainder()[slice_start..count]);
                        slice_start = count + 1;
                    }
                    ';' | '\\' | ' ' | '\t' | '\r' | '\n' | '(' | ')' | '[' | ']' | '{' | '}'
                    | '"' | '\'' => break,
                    _ => {}
                }
            }
            count += c.len_utf8()
        }
        err |= slice_start == count;
        out.push(&lex.remainder()[slice_start..count]);
        lex.bump(count);
        if err {
            return Err(LexerError::EmptyPathSegment);
        } else {
            return Ok(out);
        }
    }
    #[inline]
    fn invalid_float<'a>(_: &mut Lexer<'a, Token<'a>>) -> Result<f64, LexerError> {
        Err(LexerError::InvalidFloat)
    }
    #[inline]
    fn parse_num<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<i64, LexerError> {
        parse_num_inner(lex.slice())
    }
    #[inline]
    fn parse_num_neg<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<i64, LexerError> {
        parse_num_inner(&lex.slice()[1..]).map(|o| o * -1)
    }
    #[inline]
    fn parse_num_inner(s: &str) -> Result<i64, LexerError> {
        let mut acc = 0i64;
        for c in s.chars() {
            match c {
                '0'..='9' => {
                    if let Some(shifted) = acc.checked_mul(10) {
                        acc = shifted;
                    } else {
                        return Err(LexerError::IntegerOverflow);
                    }
                    acc += ((c as u8) - b'0') as i64;
                }
                '_' => {}
                _ => ::core::panicking::panic("internal error: entered unreachable code"),
            }
        }
        return Ok(acc);
    }
    #[inline]
    fn parse_float<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<f64, LexerError> {
        parse_float_inner(lex.slice())
    }
    #[inline]
    fn parse_float_neg<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<f64, LexerError> {
        parse_float_inner(&lex.slice()[1..]).map(|o| o * -1.0)
    }
    #[inline]
    fn parse_float_inner(s: &str) -> Result<f64, LexerError> {
        lexical::parse(s).map_err(|_| LexerError::FloatOverflow)
    }
    fn parse_char<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<char, LexerError> {
        let c = lex
            .remainder()
            .chars()
            .next()
            .ok_or(LexerError::UnexpectedEof)?;
        lex.bump(c.len_utf8());
        return Ok(c);
    }
}
mod parser {
    use parser_helper::{LogosTokenStream, LookaheadLexer, SimpleError, Span, new_parser};
    use anyhow::{Context, Result, bail};
    use misc_utils::Stack;
    use std::{
        fmt::{Display, Formatter, Result as FmtResult},
        rc::Rc,
        error::Error,
    };
    use crate::{lexer::*, ast::*};
    pub struct MyParser<'a>(
        ::parser_helper::LookaheadLexer<1, Token<'a>, LogosTokenStream<'a, Token<'a>>, ParserData>,
    );
    impl<'a> MyParser<'a> {
        pub fn new<L: Into<LogosTokenStream<'a, Token<'a>>>>(lexer: L, data: ParserData) -> Self {
            Self(LookaheadLexer::new(lexer.into(), data))
        }
    }
    impl<'a> std::ops::Deref for MyParser<'a> {
        type Target = ::parser_helper::LookaheadLexer<
            1,
            Token<'a>,
            LogosTokenStream<'a, Token<'a>>,
            ParserData,
        >;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<'a> std::ops::DerefMut for MyParser<'a> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<'a> MyParser<'a> {
        pub fn finish(self) -> ParserData {
            self.0.finish()
        }
        pub fn parse(&mut self) -> Result<()> {
            ::core::panicking::panic("not yet implemented");
        }
        pub fn parse_expr(&mut self) -> Result<ExprId> {
            match self.peek() {
                Token::Paren(Start) => match self.peek1() {
                    Token::Ident(kw) => match *kw {
                        "def" => self.parse_def(),
                        "set" => self.parse_set(),
                        "defn" => self.parse_func(),
                        _ => self.parse_call(),
                    },
                    _ => self.parse_call(),
                },
                _ => self.parse_primitive(),
            }
        }
        pub fn parse_def(&mut self) -> Result<ExprId> {
            self.paren_start()?;
            self.match_ident("def")?;
            let name = self.ident()?;
            let expr = self.parse_expr()?;
            self.paren_end()?;
            return Ok(self.expr(Expr::DefVar(name, expr)));
        }
        pub fn parse_set(&mut self) -> Result<ExprId> {
            self.paren_start()?;
            self.match_ident("set")?;
            let name = self.ident()?;
            let expr = self.parse_expr()?;
            self.paren_end()?;
            return Ok(self.expr(Expr::SetVar(name, expr)));
        }
        pub fn parse_func(&mut self) -> Result<ExprId> {
            self.paren_start()?;
            self.match_ident("defn")?;
            let name = self.ident();
            ::core::panicking::panic("not yet implemented");
        }
        pub fn parse_call(&mut self) -> Result<ExprId> {
            self.paren_start()?;
            let first = self.parse_expr()?;
            let mut others = Vec::new();
            while !self.try_paren_end() {
                others.push(self.parse_expr()?);
            }
            return Ok(self.expr(Expr::Call(first, others)));
        }
        pub fn parse_primitive(&mut self) -> Result<ExprId> {
            match self.next() {
                Token::Ident(name) => {
                    let ident = self.intern(name);
                    Ok(self.expr(Expr::GetVar(ident)))
                }
                Token::Number(n) => Ok(self.expr(Expr::Number(n))),
                Token::Float(n) => Ok(self.expr(Expr::Float(n))),
                Token::String(s) => Ok(self.expr(Expr::String(Rc::new(s)))),
                Token::Char(c) => Ok(self.expr(Expr::Char(c))),
                Token::HashLiteral("#t") => Ok(self.expr(Expr::Bool(true))),
                Token::HashLiteral("#f") => Ok(self.expr(Expr::Bool(false))),
                _ => {
                    return ::anyhow::__private::Err({
                        use ::anyhow::__private::kind::*;
                        let error = match self.error("Expected primitive expression") {
                            error => (&error).anyhow_kind().new(error),
                        };
                        error
                    })
                }
            }
        }
    }
    impl<'a> MyParser<'a> {
        fn expr(&mut self, expr: Expr) -> ExprId {
            self.user_data.exprs.insert(expr)
        }
        #[inline]
        fn peek(&mut self) -> &Token<'a> {
            self.lookahead(0)
        }
        fn peek1(&mut self) -> &Token<'a> {
            self.lookahead(1)
        }
        #[inline]
        fn peek_span(&mut self) -> Span {
            self.lookahead_span(0)
        }
        #[inline]
        fn next(&mut self) -> Token<'a> {
            self.take_token()
        }
        #[inline]
        fn error(&mut self, msg: impl Into<String>) -> SimpleError<String> {
            self.0.error(msg)
        }
        #[inline]
        fn intern(&mut self, s: &str) -> Ident {
            self.user_data.interner.intern(s)
        }
        fn ident(&mut self) -> Result<Ident> {
            match self.take_token() {
                Token::Ident(s) => Ok(self.intern(s)),
                _ => {
                    return ::anyhow::__private::Err({
                        use ::anyhow::__private::kind::*;
                        let error = match self.error("Expected identifier") {
                            error => (&error).anyhow_kind().new(error),
                        };
                        error
                    })
                }
            }
        }
        fn match_ident(&mut self, to_match: &str) -> Result<()> {
            match self.take_token() {
                Token::Ident(s) => {
                    if s != to_match {
                        return ::anyhow::__private::Err({
                            use ::anyhow::__private::kind::*;
                            let error = match self.error({
                                let res = ::alloc::fmt::format(format_args!(
                                    "Expected identifier `{0}`, but got `{1}`",
                                    to_match, s
                                ));
                                res
                            }) {
                                error => (&error).anyhow_kind().new(error),
                            };
                            error
                        });
                    } else {
                        Ok(())
                    }
                }
                _ => {
                    return ::anyhow::__private::Err({
                        use ::anyhow::__private::kind::*;
                        let error = match self.error("Expected identifier") {
                            error => (&error).anyhow_kind().new(error),
                        };
                        error
                    })
                }
            }
        }
        fn paren_start(&mut self) -> Result<()> {
            match self.take_token() {
                Token::Paren(Start) => Ok(()),
                _ => {
                    return ::anyhow::__private::Err({
                        use ::anyhow::__private::kind::*;
                        let error = match self.error("Expected `(`") {
                            error => (&error).anyhow_kind().new(error),
                        };
                        error
                    })
                }
            }
        }
        fn paren_end(&mut self) -> Result<()> {
            match self.take_token() {
                Token::Paren(End) => Ok(()),
                _ => {
                    return ::anyhow::__private::Err({
                        use ::anyhow::__private::kind::*;
                        let error = match self.error("Expected `)`") {
                            error => (&error).anyhow_kind().new(error),
                        };
                        error
                    })
                }
            }
        }
        fn try_paren_end(&mut self) -> bool {
            match self.peek() {
                Token::Paren(End) => {
                    self.take_token();
                    true
                }
                _ => false,
            }
        }
    }
    pub struct ParserData {
        pub interner: Interner,
        pub exprs: ExprStore,
        pub funcs: FunctionStore,
    }
}
mod ast {
    use indexmap::IndexSet;
    use rustc_hash::FxHashMap;
    use misc_utils::{Key, define_keys};
    use std::rc::Rc;
    use crate::misc::*;
    pub type FunctionStore = IndexedItemStore<FnId, Function>;
    pub type ExprStore = IndexedItemStore<ExprId, Expr>;
    pub type IdentMap<T> = FxHashMap<Ident, T>;
    pub enum Expr {
        Begin(Vec<ExprId>),
        /// Define a var in the given scope
        DefVar(Ident, ExprId),
        /// Set a variable with the given expr's data
        SetVar(Ident, ExprId),
        /// Get the data in a variable
        GetVar(Ident),
        Cond {
            branches: Vec<CondBranch>,
            default: Option<ExprId>,
        },
        /// Get the function with the given id
        Function(FnId),
        /// Call the data with the args
        Call(ExprId, Vec<ExprId>),
        /// Call a method with the args. This is always generated on the first pass over the AST so
        /// we don't have special-cases all over the parser.
        Method(ExprId, Ident, Vec<ExprId>),
        /// Get the field on a given object
        GetField {
            name: Ident,
            data: ExprId,
        },
        /// Set the field on a given object
        SetField {
            obj: ExprId,
            name: Ident,
            data: ExprId,
        },
        String(Rc<String>),
        Number(i64),
        Float(f64),
        Char(char),
        Bool(bool),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Expr {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                Expr::Begin(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Begin", &__self_0)
                }
                Expr::DefVar(__self_0, __self_1) => {
                    ::core::fmt::Formatter::debug_tuple_field2_finish(
                        f, "DefVar", __self_0, &__self_1,
                    )
                }
                Expr::SetVar(__self_0, __self_1) => {
                    ::core::fmt::Formatter::debug_tuple_field2_finish(
                        f, "SetVar", __self_0, &__self_1,
                    )
                }
                Expr::GetVar(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "GetVar", &__self_0)
                }
                Expr::Cond {
                    branches: __self_0,
                    default: __self_1,
                } => ::core::fmt::Formatter::debug_struct_field2_finish(
                    f, "Cond", "branches", __self_0, "default", &__self_1,
                ),
                Expr::Function(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Function", &__self_0)
                }
                Expr::Call(__self_0, __self_1) => {
                    ::core::fmt::Formatter::debug_tuple_field2_finish(
                        f, "Call", __self_0, &__self_1,
                    )
                }
                Expr::Method(__self_0, __self_1, __self_2) => {
                    ::core::fmt::Formatter::debug_tuple_field3_finish(
                        f, "Method", __self_0, __self_1, &__self_2,
                    )
                }
                Expr::GetField {
                    name: __self_0,
                    data: __self_1,
                } => ::core::fmt::Formatter::debug_struct_field2_finish(
                    f, "GetField", "name", __self_0, "data", &__self_1,
                ),
                Expr::SetField {
                    obj: __self_0,
                    name: __self_1,
                    data: __self_2,
                } => ::core::fmt::Formatter::debug_struct_field3_finish(
                    f, "SetField", "obj", __self_0, "name", __self_1, "data", &__self_2,
                ),
                Expr::String(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "String", &__self_0)
                }
                Expr::Number(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Number", &__self_0)
                }
                Expr::Float(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Float", &__self_0)
                }
                Expr::Char(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Char", &__self_0)
                }
                Expr::Bool(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Bool", &__self_0)
                }
            }
        }
    }
    pub enum Function {
        Single {
            captures: Vec<Ident>,
            func: FunctionVariant,
        },
        Multiple {
            catures: Vec<Ident>,
            variants: Vec<FunctionVariant>,
        },
    }
    #[repr(transparent)]
    pub struct FnId(pub usize);
    #[automatically_derived]
    impl ::core::fmt::Debug for FnId {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "FnId", &&self.0)
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for FnId {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for FnId {
        #[inline]
        fn eq(&self, other: &FnId) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for FnId {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<usize>;
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for FnId {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for FnId {}
    #[automatically_derived]
    impl ::core::clone::Clone for FnId {
        #[inline]
        fn clone(&self) -> FnId {
            let _: ::core::clone::AssertParamIsClone<usize>;
            *self
        }
    }
    impl ::misc_utils::Key for FnId {
        fn from_id(id: usize) -> Self {
            FnId(id)
        }
        fn id(&self) -> usize {
            self.0
        }
    }
    impl FnId {
        pub fn invalid() -> Self {
            Self(usize::MAX)
        }
    }
    #[repr(transparent)]
    pub struct ExprId(pub usize);
    #[automatically_derived]
    impl ::core::fmt::Debug for ExprId {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "ExprId", &&self.0)
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ExprId {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ExprId {
        #[inline]
        fn eq(&self, other: &ExprId) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for ExprId {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<usize>;
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for ExprId {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for ExprId {}
    #[automatically_derived]
    impl ::core::clone::Clone for ExprId {
        #[inline]
        fn clone(&self) -> ExprId {
            let _: ::core::clone::AssertParamIsClone<usize>;
            *self
        }
    }
    impl ::misc_utils::Key for ExprId {
        fn from_id(id: usize) -> Self {
            ExprId(id)
        }
        fn id(&self) -> usize {
            self.0
        }
    }
    impl ExprId {
        pub fn invalid() -> Self {
            Self(usize::MAX)
        }
    }
    #[repr(transparent)]
    pub struct Ident(pub usize);
    #[automatically_derived]
    impl ::core::fmt::Debug for Ident {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Ident", &&self.0)
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Ident {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Ident {
        #[inline]
        fn eq(&self, other: &Ident) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for Ident {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<usize>;
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for Ident {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Ident {}
    #[automatically_derived]
    impl ::core::clone::Clone for Ident {
        #[inline]
        fn clone(&self) -> Ident {
            let _: ::core::clone::AssertParamIsClone<usize>;
            *self
        }
    }
    impl ::misc_utils::Key for Ident {
        fn from_id(id: usize) -> Self {
            Ident(id)
        }
        fn id(&self) -> usize {
            self.0
        }
    }
    impl Ident {
        pub fn invalid() -> Self {
            Self(usize::MAX)
        }
    }
    pub struct Interner(IndexSet<String>);
    #[automatically_derived]
    impl ::core::fmt::Debug for Interner {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Interner", &&self.0)
        }
    }
    impl Interner {
        pub fn intern(&mut self, s: &str) -> Ident {
            if let Some(idx) = self.0.get_index_of(s) {
                return Ident::from_id(idx);
            }
            let (idx, _) = self.0.insert_full(s.to_string());
            return Ident::from_id(idx);
        }
        #[inline]
        pub fn get(&self, id: Ident) -> &str {
            self.0.get_index(id.0).unwrap()
        }
    }
    pub struct CondBranch {
        pub condition: ExprId,
        pub body: ExprId,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for CondBranch {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "CondBranch",
                "condition",
                &self.condition,
                "body",
                &&self.body,
            )
        }
    }
    pub struct FunctionVariant {
        pub params: Vec<Ident>,
        pub block: ExprId,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for FunctionVariant {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "FunctionVariant",
                "params",
                &self.params,
                "block",
                &&self.block,
            )
        }
    }
}
mod interpreter {
    use data::DataRef;
    use object::Object;
    mod object {
        use anyhow::Result;
        use crate::ast::{IdentMap, Ident, FnId};
        use super::{Primitive, GcTracer};
        pub trait Object {
            type ObjectBundle: Object;
            fn get(&self, name: Ident) -> Result<Primitive<Self::ObjectBundle>>;
            fn set(&mut self, name: Ident, data: Primitive<Self::ObjectBundle>) -> Result<()>;
            fn call(
                &mut self,
                args: Vec<Primitive<Self::ObjectBundle>>,
            ) -> Result<CallReturn<Self::ObjectBundle>>;
            fn method(
                &mut self,
                name: Ident,
                args: Vec<Primitive<Self::ObjectBundle>>,
            ) -> Result<CallReturn<Self::ObjectBundle>>;
            fn trace<T: GcTracer>(&mut self, tracer: &mut T);
        }
        pub enum CallReturn<O: Object> {
            CallFn(FnId, Vec<Primitive<O>>),
            Data(Primitive<O>),
        }
        pub enum MyObjects {
            BaseObject(BaseObject<MyObjects>),
            BaseObject2(BaseObject2<MyObjects>),
        }
        impl Object for MyObjects {
            type ObjectBundle = Self;
            #[inline]
            fn get(&self, name: Ident) -> Result<Primitive<MyObjects>> {
                match self {
                    BaseObject(variant) => variant.get(name),
                    BaseObject2(variant) => variant.get(name),
                }
            }
            #[inline]
            fn set(&mut self, name: Ident, data: Primitive<MyObjects>) -> Result<()> {
                match self {
                    BaseObject(variant) => variant.set(name, data),
                    BaseObject2(variant) => variant.set(name, data),
                }
            }
            #[inline]
            fn call(&mut self, args: Vec<Primitive<MyObjects>>) -> Result<CallReturn<Self>> {
                match self {
                    BaseObject(variant) => variant.call(args),
                    BaseObject2(variant) => variant.call(args),
                }
            }
            #[inline]
            fn method(
                &mut self,
                name: Ident,
                args: Vec<Primitive<MyObjects>>,
            ) -> Result<CallReturn<Self>> {
                match self {
                    BaseObject(variant) => variant.call(name, args),
                    BaseObject2(variant) => variant.call(name, args),
                }
            }
            #[inline]
            fn trace<T: GcTracer>(&mut self, tracer: &mut T) {
                match self {
                    BaseObject(variant) => variant.trace(tracer),
                    BaseObject2(variant) => variant.trace(tracer),
                }
            }
        }
        pub struct BaseObject<O: Object>(IdentMap<Primitive<O>>);
        impl<O: Object> Object for BaseObject<O> {
            type ObjectBundle = O;
            fn get(&self, name: Ident) -> Result<Primitive<Self::ObjectBundle>> {
                ::core::panicking::panic("not yet implemented");
            }
            fn set(&mut self, name: Ident, data: Primitive<Self::ObjectBundle>) -> Result<()> {
                ::core::panicking::panic("not yet implemented");
            }
            fn call(
                &mut self,
                args: Vec<Primitive<Self::ObjectBundle>>,
            ) -> Result<CallReturn<Self::ObjectBundle>> {
                ::core::panicking::panic("not yet implemented");
            }
            fn method(
                &mut self,
                name: Ident,
                args: Vec<Primitive<Self::ObjectBundle>>,
            ) -> Result<CallReturn<Self::ObjectBundle>> {
                ::core::panicking::panic("not yet implemented");
            }
            fn trace<T: GcTracer>(&mut self, tracer: &mut T) {
                ::core::panicking::panic("not yet implemented");
            }
        }
        pub struct BaseObject2<O: Object>(IdentMap<Primitive<O>>);
        impl<O: Object> Object for BaseObject2<O> {
            type ObjectBundle = O;
            fn get(&self, name: Ident) -> Result<Primitive<Self::ObjectBundle>> {
                ::core::panicking::panic("not yet implemented");
            }
            fn set(&mut self, name: Ident, data: Primitive<Self::ObjectBundle>) -> Result<()> {
                ::core::panicking::panic("not yet implemented");
            }
            fn call(
                &mut self,
                args: Vec<Primitive<Self::ObjectBundle>>,
            ) -> Result<CallReturn<Self::ObjectBundle>> {
                ::core::panicking::panic("not yet implemented");
            }
            fn method(
                &mut self,
                name: Ident,
                args: Vec<Primitive<Self::ObjectBundle>>,
            ) -> Result<CallReturn<Self::ObjectBundle>> {
                ::core::panicking::panic("not yet implemented");
            }
            fn trace<T: GcTracer>(&mut self, tracer: &mut T) {
                ::core::panicking::panic("not yet implemented");
            }
        }
    }
    mod data {
        use std::ptr::NonNull;
        use super::object::*;
        pub enum Data<O: Object> {
            Base(BaseObject<O>),
            Custom(O),
        }
        pub struct DataRef<O: Object>(NonNull<DataBox<O>>);
        pub struct DataBox<O: Object> {
            data: Data<O>,
        }
        pub struct Gc {}
    }
    pub trait GcTracer {
        fn trace(&mut self, ptr: ());
    }
    pub enum Primitive<O: Object> {
        Data(DataRef<O>),
        Number(i64),
        Float(f64),
        Char(char),
        Bool(bool),
    }
}
fn main() {
    let source = read_to_string("example.eka").unwrap();
    for tok in lexer::Token::lexer(&source) {
        match tok {
            tmp => {
                {
                    ::std::io::_eprint(format_args!(
                        "[{0}:{1}:{2}] {3} = {4:#?}\n",
                        "src/main.rs", 19u32, 9u32, "tok", &tmp
                    ));
                };
                tmp
            }
        }
        .ok();
    }
}
