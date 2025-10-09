use std::cell::RefCell;
use std::ops::RangeTo;

use bstr::ByteSlice;
use winnow::ascii::digit1;
use winnow::combinator::{
    alt, delimited, empty, opt, preceded, repeat, separated, terminated,
};
use winnow::error::ParserError;
use winnow::{ModalResult, Parser};

use super::{Format, Fragments, Group, List, Modifier, Value};
use crate::matcher::occurrence::parse_occurrence_matcher;
use crate::matcher::subfield::parser::parse_subfield_matcher;
use crate::matcher::tag::parse_tag_matcher;
use crate::parser::{parse_string, parse_subfield_codes, ws};

pub(crate) fn parse_format(i: &mut &[u8]) -> ModalResult<Format> {
    (
        parse_tag_matcher,
        parse_occurrence_matcher,
        delimited(
            ws('{'),
            (
                parse_fragments,
                opt(preceded(ws('|'), parse_subfield_matcher)),
            ),
            ws('}'),
        ),
    )
        .with_taken()
        .map(|((t, o, (f, s)), r)| {
            let raw_format = r.to_str().unwrap().trim().to_string();
            Format {
                tag_matcher: t,
                occurrence_matcher: o,
                subfield_matcher: s,
                fragments: f,
                raw_format,
            }
        })
        .parse_next(i)
}

fn parse_fragments(i: &mut &[u8]) -> ModalResult<Fragments> {
    alt((
        ws(parse_list).map(Fragments::List),
        ws(parse_group).map(Fragments::Group),
        ws(parse_value).map(Fragments::Value),
    ))
    .parse_next(i)
}

thread_local! {
    pub static GROUP_LEVEL: RefCell<u32> = const { RefCell::new(0) };
}

fn group_level_inc(i: &mut &[u8]) -> ModalResult<()> {
    GROUP_LEVEL.with(|level| {
        *level.borrow_mut() += 1;
        if *level.borrow() >= 32 {
            Err(winnow::error::ErrMode::from_input(i))
        } else {
            Ok(())
        }
    })
}

fn group_level_dec() {
    GROUP_LEVEL.with(|level| {
        *level.borrow_mut() -= 1;
    })
}

fn parse_group(i: &mut &[u8]) -> ModalResult<Group> {
    (
        terminated(ws('('), group_level_inc),
        parse_modifier,
        alt((
            ws(parse_list).map(Fragments::List),
            ws(parse_value).map(Fragments::Value),
        )),
        ws(')').map(|_| group_level_dec()),
        alt((
            preceded(
                "..",
                digit1
                    .verify_map(|s: &[u8]| s.to_str().ok())
                    .verify_map(|s: &str| s.parse::<usize>().ok()),
            ),
            "..".value(usize::MAX),
            empty.value(usize::MAX),
        )),
    )
        .map(|(_, modifier, fragments, _, end)| Group {
            fragments: Box::new(fragments),
            bounds: RangeTo { end },
            modifier: modifier.unwrap_or_default(),
        })
        .parse_next(i)
}

fn parse_value(i: &mut &[u8]) -> ModalResult<Value> {
    (
        ws(parse_modifier),
        opt(ws(parse_string).map(|s| {
            // SAFETY: `parse_string` returns a valid string and
            // therefore it's safe to unwrap the result.
            s.to_str().unwrap().to_string()
        })),
        parse_subfield_codes,
        alt((
            preceded(
                "..",
                digit1
                    .verify_map(|s: &[u8]| s.to_str().ok())
                    .verify_map(|s: &str| s.parse::<usize>().ok()),
            ),
            "..".value(usize::MAX),
            empty.value(1),
        )),
        opt(ws(parse_string).map(|s| {
            // SAFETY: `parse_string` returns a valid string and
            // therefore it's safe to unwrap the result.
            s.to_str().unwrap().to_string()
        })),
    )
        .map(|(modifier, prefix, codes, end, suffix)| Value {
            modifier: modifier.unwrap_or_default(),
            prefix,
            codes,
            suffix,
            bounds: RangeTo { end },
        })
        .parse_next(i)
}

#[inline]
fn parse_list(i: &mut &[u8]) -> ModalResult<List> {
    alt((parse_list_cons, parse_list_and_then)).parse_next(i)
}

fn parse_list_cons(i: &mut &[u8]) -> ModalResult<List> {
    separated(
        2..,
        alt((
            parse_list_and_then.map(Fragments::List),
            parse_group.map(Fragments::Group),
            parse_value.map(Fragments::Value),
        )),
        ws("<*>"),
    )
    .map(List::Cons)
    .parse_next(i)
}

fn parse_list_and_then(i: &mut &[u8]) -> ModalResult<List> {
    separated(
        2..,
        alt((
            parse_group.map(Fragments::Group),
            parse_value.map(Fragments::Value),
        )),
        ws("<$>"),
    )
    .map(List::AndThen)
    .parse_next(i)
}

fn parse_modifier(i: &mut &[u8]) -> ModalResult<Option<Modifier>> {
    opt(preceded(
        '?',
        repeat(1.., alt(('l', 'u', 't', 'w', 'o'))).map(
            |codes: Vec<_>| {
                let mut modifier = Modifier::default();
                if codes.contains(&'o') {
                    modifier.strip_overread_char(true);
                }

                if codes.contains(&'l') {
                    modifier.lowercase(true);
                }

                if codes.contains(&'u') {
                    modifier.uppercase(true);
                }

                if codes.contains(&'w') {
                    modifier.remove_ws(true);
                }

                if codes.contains(&'t') {
                    modifier.trim(true);
                }

                modifier
            },
        ),
    ))
    .parse_next(i)
}

#[cfg(test)]
mod tests {
    use smallvec::SmallVec;

    use super::*;
    use crate::primitives::SubfieldCode;

    type TestResult = anyhow::Result<()>;

    #[test]
    fn test_parse_modifier() -> TestResult {
        assert_eq!(
            parse_modifier.parse(b"?o").unwrap().unwrap(),
            Modifier {
                strip_overread_char: true,
                ..Default::default()
            }
        );

        assert_eq!(
            parse_modifier.parse(b"?l").unwrap().unwrap(),
            Modifier {
                lowercase: true,
                ..Default::default()
            }
        );

        assert_eq!(
            parse_modifier.parse(b"?u").unwrap().unwrap(),
            Modifier {
                uppercase: true,
                ..Default::default()
            }
        );

        assert_eq!(
            parse_modifier.parse(b"?w").unwrap().unwrap(),
            Modifier {
                remove_ws: true,
                ..Default::default()
            }
        );

        assert_eq!(
            parse_modifier.parse(b"?t").unwrap().unwrap(),
            Modifier {
                trim: true,
                ..Default::default()
            }
        );

        assert_eq!(
            parse_modifier.parse(b"?luwto").unwrap().unwrap(),
            Modifier {
                strip_overread_char: true,
                lowercase: true,
                uppercase: true,
                remove_ws: true,
                trim: true,
            }
        );

        assert_eq!(
            parse_modifier.parse(b"?luwtoluwto").unwrap().unwrap(),
            Modifier {
                strip_overread_char: true,
                lowercase: true,
                uppercase: true,
                remove_ws: true,
                trim: true,
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_value() -> TestResult {
        assert_eq!(
            parse_value.parse(b"a").unwrap(),
            Value {
                prefix: None,
                codes: SmallVec::from_vec(vec![SubfieldCode::new(
                    'a'
                )?]),
                bounds: RangeTo { end: 1 },
                suffix: None,
                modifier: Modifier::default(),
            }
        );

        assert_eq!(
            parse_value.parse(b"a..2").unwrap(),
            Value {
                prefix: None,
                codes: SmallVec::from_vec(vec![SubfieldCode::new(
                    'a'
                )?]),
                bounds: RangeTo { end: 2 },
                suffix: None,
                modifier: Modifier::default(),
            }
        );

        assert_eq!(
            parse_value.parse(b"a..").unwrap(),
            Value {
                prefix: None,
                codes: SmallVec::from_vec(vec![SubfieldCode::new(
                    'a'
                )?]),
                bounds: RangeTo { end: usize::MAX },
                suffix: None,
                modifier: Modifier::default(),
            }
        );

        assert_eq!(
            parse_value.parse(b"a..2 'def'").unwrap(),
            Value {
                prefix: None,
                codes: SmallVec::from_vec(vec![SubfieldCode::new(
                    'a'
                )?]),
                bounds: RangeTo { end: 2 },
                suffix: Some("def".to_string()),
                modifier: Modifier::default(),
            }
        );

        assert_eq!(
            parse_value.parse(b"'abc' a..2 'def'").unwrap(),
            Value {
                prefix: Some("abc".to_string()),
                codes: SmallVec::from_vec(vec![SubfieldCode::new(
                    'a'
                )?]),
                bounds: RangeTo { end: 2 },
                suffix: Some("def".to_string()),
                modifier: Modifier::default(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_group() -> TestResult {
        assert_eq!(
            parse_group.parse(b"(a)").unwrap(),
            Group {
                fragments: Box::new(Fragments::Value(Value {
                    prefix: None,
                    codes: SmallVec::from_vec(vec![SubfieldCode::new(
                        'a'
                    )?]),
                    bounds: RangeTo { end: 1 },
                    suffix: None,
                    modifier: Modifier::default(),
                })),
                bounds: RangeTo { end: usize::MAX },
                modifier: Modifier::default(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_list_cons() -> TestResult {
        assert_eq!(
            parse_list_cons.parse(b"a <*> b").unwrap(),
            List::Cons(vec![
                Fragments::Value(Value {
                    prefix: None,
                    codes: SmallVec::from_vec(vec![SubfieldCode::new(
                        'a'
                    )?]),
                    bounds: RangeTo { end: 1 },
                    suffix: None,
                    modifier: Modifier::default(),
                }),
                Fragments::Value(Value {
                    prefix: None,
                    codes: SmallVec::from_vec(vec![SubfieldCode::new(
                        'b'
                    )?]),
                    bounds: RangeTo { end: 1 },
                    suffix: None,
                    modifier: Modifier::default(),
                })
            ])
        );

        assert_eq!(
            parse_list_cons.parse(b"a <*> b <*> c").unwrap(),
            List::Cons(vec![
                Fragments::Value(Value {
                    prefix: None,
                    codes: SmallVec::from_vec(vec![SubfieldCode::new(
                        'a'
                    )?]),
                    bounds: RangeTo { end: 1 },
                    suffix: None,
                    modifier: Modifier::default(),
                }),
                Fragments::Value(Value {
                    prefix: None,
                    codes: SmallVec::from_vec(vec![SubfieldCode::new(
                        'b'
                    )?]),
                    bounds: RangeTo { end: 1 },
                    suffix: None,
                    modifier: Modifier::default(),
                }),
                Fragments::Value(Value {
                    prefix: None,
                    codes: SmallVec::from_vec(vec![SubfieldCode::new(
                        'c'
                    )?]),
                    bounds: RangeTo { end: 1 },
                    suffix: None,
                    modifier: Modifier::default(),
                })
            ])
        );

        assert_eq!(
            parse_list_cons.parse(b"a <*> (b <*> c)").unwrap(),
            List::Cons(vec![
                Fragments::Value(Value {
                    prefix: None,
                    codes: SmallVec::from_vec(vec![SubfieldCode::new(
                        'a'
                    )?]),
                    bounds: RangeTo { end: 1 },
                    suffix: None,
                    modifier: Modifier::default(),
                }),
                Fragments::Group(Group {
                    fragments: Box::new(Fragments::List(List::Cons(
                        vec![
                            Fragments::Value(Value {
                                prefix: None,
                                codes: SmallVec::from_vec(vec![
                                    SubfieldCode::new('b')?
                                ]),
                                bounds: RangeTo { end: 1 },
                                suffix: None,
                                modifier: Modifier::default(),
                            }),
                            Fragments::Value(Value {
                                prefix: None,
                                codes: SmallVec::from_vec(vec![
                                    SubfieldCode::new('c')?
                                ]),
                                bounds: RangeTo { end: 1 },
                                suffix: None,
                                modifier: Modifier::default(),
                            })
                        ]
                    ))),
                    bounds: RangeTo { end: usize::MAX },
                    modifier: Modifier::default()
                }),
            ])
        );

        Ok(())
    }

    #[test]
    fn test_parse_list_and_then() -> TestResult {
        assert_eq!(
            parse_list_and_then.parse(b"a <$> b").unwrap(),
            List::AndThen(vec![
                Fragments::Value(Value {
                    prefix: None,
                    codes: SmallVec::from_vec(vec![SubfieldCode::new(
                        'a'
                    )?]),
                    bounds: RangeTo { end: 1 },
                    suffix: None,
                    modifier: Modifier::default(),
                }),
                Fragments::Value(Value {
                    prefix: None,
                    codes: SmallVec::from_vec(vec![SubfieldCode::new(
                        'b'
                    )?]),
                    bounds: RangeTo { end: 1 },
                    suffix: None,
                    modifier: Modifier::default(),
                })
            ])
        );

        assert_eq!(
            parse_list_and_then.parse(b"a <$> (b <*> c)").unwrap(),
            List::AndThen(vec![
                Fragments::Value(Value {
                    prefix: None,
                    codes: SmallVec::from_vec(vec![SubfieldCode::new(
                        'a'
                    )?]),
                    bounds: RangeTo { end: 1 },
                    suffix: None,
                    modifier: Modifier::default(),
                }),
                Fragments::Group(Group {
                    fragments: Box::new(Fragments::List(List::Cons(
                        vec![
                            Fragments::Value(Value {
                                prefix: None,
                                codes: SmallVec::from_vec(vec![
                                    SubfieldCode::new('b')?
                                ]),
                                bounds: RangeTo { end: 1 },
                                suffix: None,
                                modifier: Modifier::default(),
                            }),
                            Fragments::Value(Value {
                                prefix: None,
                                codes: SmallVec::from_vec(vec![
                                    SubfieldCode::new('c')?
                                ]),
                                bounds: RangeTo { end: 1 },
                                suffix: None,
                                modifier: Modifier::default(),
                            })
                        ]
                    ))),
                    bounds: RangeTo { end: usize::MAX },
                    modifier: Modifier::default()
                }),
            ])
        );

        Ok(())
    }

    #[test]
    fn test_parse_format() {
        let inputs = vec![
            "028[A@]{[aP] <$> (', ' d <*> ' ' c) | !U? || U == 'Latn'}",
            "042A{'https://d-nb.info/standards/vocab/gnd/gnd-sc#' a }",
            "006Y{(?w 'https://isni.org/isni/' 0) | S == 'isni' }",
            "029A{a <$> (' (' g ')' <*> ' / ' [xb] <*> ', ' n)..1}",
            "029A{a <$> (' (' g ')' <*> ' / ' [xb] <*> ', ' n)..2}",
            "029A{a <$> (' (' g ')' <*> ' / ' [xb] <*> ', ' n)..3}",
            "029A{a <$> (' (' g ')' <*> ' / ' [xb] <*> ', ' n)..}",
            "029A{a <$> (' (' g ')' <*> ' / ' [xb] <*> ', ' n)}",
            "041A{ x.. <$> (a <*> b)..2 <$> (c <*> d).. | a? }",
        ];

        for i in inputs {
            assert!(parse_format.parse(i.as_bytes()).is_ok());
        }
    }

    mod regressions {
        use super::*;

        #[test]
        fn test_invalid_byte_seq() {
            let input = vec![
                48, 50, 50, 88, 13, 13, 13, 13, 13, 10, 123, 40, 40,
                40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40,
                40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40,
                40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40,
                40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40,
                40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40,
                40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40,
                40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40,
            ];

            assert!(parse_format.parse(&input).is_err());

            let input = vec![
                48, 50, 50, 88, 13, 13, 13, 13, 13, 13, 10, 123, 13,
                40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40,
                40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40,
                40, 40, 40, 40, 40, 13, 13, 13, 88, 13, 13, 19, 13, 34,
                10, 49, 50, 46, 48, 50, 50, 88, 13, 13, 10, 13, 13, 13,
                123, 13, 13, 34, 10, 9, 10, 13, 13, 13, 13, 13, 88,
            ];

            assert!(parse_format.parse(&input).is_err());
        }
    }
}
