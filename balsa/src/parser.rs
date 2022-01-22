/// Represents a parsed token.
#[derive(Debug, PartialEq)]
pub(crate) struct Parsed<T> {
    pub(crate) start_pos: i32,
    pub(crate) end_pos: i32,
    pub(crate) token: T,
}

/// Represents a parsing failure.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum ParseError {
    NotMatched,
    MalformedInput(i32),
}

/// The result of running a [`Parser`] on an input.
pub(crate) type ParseResult<'a, T> = Result<(&'a str, Parsed<T>), ParseError>;

/// This trait describes a generic string parser.
pub(crate) trait Parser<'a, T>: 'a {
    fn parse(&self, pos: i32, input: &'a str) -> ParseResult<'a, T>;
}

/// A wrapper struct that holds a [`Parser<'a, T>`] in a [`Box`].
pub(crate) struct ParserB<'a, T: 'a> {
    parser: Box<dyn Parser<'a, T>>,
}

impl<'a, T> ParserB<'a, T> {
    pub(crate) fn new<P>(parser: P) -> ParserB<'a, T>
    where
        P: Parser<'a, T>,
    {
        ParserB {
            parser: Box::new(parser),
        }
    }
}

impl<'a, T> Parser<'a, T> for ParserB<'a, T>
where
    T: 'a,
{
    fn parse(&self, pos: i32, input: &'a str) -> ParseResult<'a, T> {
        self.parser.parse(pos, input)
    }
}

impl<'a, T> Parser<'a, T> for &'a ParserB<'a, T>
where
    T: 'a,
{
    fn parse(&self, pos: i32, input: &'a str) -> ParseResult<'a, T> {
        self.parser.parse(pos, input)
    }
}

/// This trait allows two types to be combined into one.
pub(crate) trait Combinable<T, O> {
    /// Combines two types into type `T`.
    fn combine(&self, with: T) -> O;
}

impl Combinable<char, String> for char {
    /// Combines two chars into a String.
    fn combine(&self, with: char) -> String {
        let mut s = String::new();
        s.push(*self);
        s.push(with);
        s
    }
}

impl Combinable<char, String> for String {
    /// Combines a String and a char into a String.
    fn combine(&self, with: char) -> String {
        let mut s = String::new();
        s.push_str(self);
        s.push(with);
        s
    }
}

impl Combinable<String, String> for String {
    fn combine(&self, with: String) -> String {
        format!("{}{}", self, with)
    }
}

impl<T> Combinable<Vec<T>, Vec<T>> for T
where
    T: Clone,
{
    fn combine(&self, with: Vec<T>) -> Vec<T> {
        let mut extended_vec = vec![self.clone()];
        extended_vec.extend(with);

        extended_vec
    }
}

impl<T> Combinable<Option<Vec<T>>, Vec<T>> for T
where
    T: Clone,
{
    fn combine(&self, with: Option<Vec<T>>) -> Vec<T> {
        let mut extended_vec = vec![self.clone()];
        if let Some(with) = with {
            extended_vec.extend(with);
        }

        extended_vec
    }
}

/// Allow any [`Parsed<I>`] where `I` is [`Combinable`] to be combined
/// with any other [`Parsed<O>`] where `O` is also [`Combinable`].
impl<T, I, O> Combinable<Parsed<I>, Parsed<O>> for Parsed<T>
where
    T: Combinable<I, O>,
{
    fn combine(&self, with: Parsed<I>) -> Parsed<O> {
        Parsed {
            start_pos: self.start_pos,
            end_pos: with.end_pos,
            token: self.token.combine(with.token),
        }
    }
}

/// Allow any parser function to be treated as a [`Parser`].
impl<'a, F, T> Parser<'a, T> for F
where
    F: Fn(i32, &'a str) -> ParseResult<'a, T> + 'a,
{
    fn parse(&self, pos: i32, input: &'a str) -> ParseResult<'a, T> {
        self(pos, input)
    }
}

trait ParserGenerator<'a, T> {
    fn gen_parser(&self) -> ParserB<'a, T>;
}

/// Allow any parser function closure to be treated as a [`Parser`].
impl<'a, C, T, O> ParserGenerator<'a, T> for C
where
    C: Fn() -> O,
    O: Parser<'a, T>,
{
    fn gen_parser(&self) -> ParserB<'a, T> {
        ParserB::new(self())
    }
}

/// Maps a [`Parser<'a, T>`] to a [`Parser<'a, O>`] using the provided
/// function `F`.
pub(crate) fn fmap<'a, P, T: 'a, O: 'a, F>(parser: P, function: F) -> ParserB<'a, O>
where
    P: Parser<'a, T> + 'a,
    F: Fn(T) -> O + 'a,
{
    fmap_result(parser, move |x| Ok(function(x)))
}

/// Maps a [`Parser<'a, T>`] to a [`Parser<'a, O>`] using the provided
/// function `F` which can fail.
pub(crate) fn fmap_result<'a, P, T: 'a, O: 'a, F>(parser: P, function: F) -> ParserB<'a, O>
where
    P: Parser<'a, T> + 'a,
    F: Fn(T) -> Result<O, ParseError> + 'a,
{
    ParserB::new(move |pos: i32, input: &'a str| {
        parser.parse(pos, input).and_then(|(remainder, output)| {
            function(output.token).map(|token| {
                (
                    remainder,
                    Parsed {
                        start_pos: output.start_pos,
                        end_pos: output.end_pos,
                        token,
                    },
                )
            })
        })
    })
}

/// Creates a new [`Parser`] which chains together two parsers which have token types that are [`Combinable`].
///
/// Parses input with the `left` [`Parser`], then feeds the output into the `right` [`Parser`].
/// Finally, it combines the two `token`s with the [`Combinable`] trait and returns a single [`Parsed`].
pub(crate) fn chain<'a, L, R, LT, RT, O>(left: L, right: R) -> ParserB<'a, O>
where
    L: Parser<'a, LT> + 'a,
    R: Parser<'a, RT> + 'a,
    LT: Combinable<RT, O> + 'a,
    RT: 'a,
{
    ParserB::new(move |pos: i32, input: &'a str| {
        left.parse(pos, input).and_then(|(remainder, left_parsed)| {
            right
                .parse(left_parsed.end_pos, remainder)
                .map(|(remainder, right_parsed)| (remainder, left_parsed.combine(right_parsed)))
        })
    })
}

/// Creates a new [`Parser`] which first tries the `left` [`Parser`], returning its output on
/// success and returning the output of the `right` [`Parser`] if `left` fails.
pub(crate) fn or<'a, L, R, T: 'a>(left: L, right: R) -> ParserB<'a, T>
where
    L: Parser<'a, T> + 'a,
    R: Parser<'a, T> + 'a,
{
    ParserB::new(move |pos: i32, input: &'a str| {
        left.parse(pos, input).or_else(|_| right.parse(pos, input))
    })
}

/// Creates a new [`Parser`] which chains together two parsers using the provided `combinator`
/// function to combine the two outputs.
///
/// Parses input with the `left` [`Parser`], then feeds the output into the `right` [`Parser`].
/// Finally, it combines the two `token`s with the `combinator` function and returns a single [`Parsed`].
pub(crate) fn fmap_chain<'a, L, R, LT: 'a, RT: 'a, O: 'a, F>(
    left: L,
    right: R,
    combinator: F,
) -> ParserB<'a, O>
where
    L: Parser<'a, LT> + 'a,
    R: Parser<'a, RT> + 'a,
    F: Fn(LT, RT) -> O + 'a,
{
    fmap_result_chain(left, right, move |x, y| Ok(combinator(x, y)))
}

/// Creates a new [`Parser`] which chains together two parsers using the provided `combinator`
/// function to combine the two outputs into a [`Result<O, ParseError>`].
///
/// Parses input with the `left` [`Parser`], then feeds the output into the `right` [`Parser`].
/// Finally, it combines the two `token`s with the `combinator` function and returns a single [`Parsed`].
pub(crate) fn fmap_result_chain<'a, L, R, LT: 'a, RT: 'a, O: 'a, F>(
    left: L,
    right: R,
    combinator: F,
) -> ParserB<'a, O>
where
    L: Parser<'a, LT> + 'a,
    R: Parser<'a, RT> + 'a,
    F: Fn(LT, RT) -> Result<O, ParseError> + 'a,
{
    ParserB::new(move |pos: i32, input: &'a str| {
        left.parse(pos, input).and_then(|(remainder, left_parsed)| {
            right
                .parse(left_parsed.end_pos, remainder)
                .and_then(|(remainder, right_parsed)| {
                    combinator(left_parsed.token, right_parsed.token).map(|token| {
                        (
                            remainder,
                            Parsed {
                                start_pos: left_parsed.start_pos,
                                end_pos: right_parsed.end_pos,
                                token,
                            },
                        )
                    })
                })
        })
    })
}

/// Creates a new [`Parser<'a, Option<T>>`] out of a [`Parser<'a, T>`] where [`Option::None`] is
/// returned when nothing is matched rather than failing.
///
/// All other failures will still be returned as an error.
pub(crate) fn optional<'a, T: 'a, P>(parser: P) -> ParserB<'a, Option<T>>
where
    P: Parser<'a, T>,
{
    ParserB::new(
        move |pos: i32, input: &'a str| match parser.parse(pos, input) {
            Ok((remainder, parsed)) => Ok((
                remainder,
                Parsed {
                    start_pos: parsed.start_pos,
                    end_pos: parsed.end_pos,
                    token: Some(parsed.token),
                },
            )),

            Err(ParseError::NotMatched) => Ok((
                input,
                Parsed {
                    start_pos: pos,
                    end_pos: pos,
                    token: None,
                },
            )),

            Err(e) => Err(e),
        },
    )
}

/// Creates a new [`Parser`] which chains together two parsers but ignores the second (right)
/// [`Parser`]'s token output.
///
/// Parses input with the `left_p` [`Parser`], then feeds the output into the `right_p` [`Parser`].
/// Finally, it ignores the right [`Parser`]'s token and returns the left's.
pub(crate) fn left<'a, L, R, LT: 'a, RT: 'a>(left_p: L, right_p: R) -> ParserB<'a, LT>
where
    L: Parser<'a, LT> + 'a,
    R: Parser<'a, RT> + 'a,
{
    ParserB::new(move |pos: i32, input: &'a str| {
        left_p
            .parse(pos, input)
            .and_then(|(remainder, left_parsed)| {
                right_p
                    .parse(left_parsed.end_pos, remainder)
                    .map(|(remainder, right_parsed)| {
                        (
                            remainder,
                            Parsed {
                                start_pos: left_parsed.start_pos,
                                end_pos: right_parsed.end_pos,
                                token: left_parsed.token,
                            },
                        )
                    })
            })
    })
}

/// Creates a new [`Parser`] which chains together two parsers but ignores the first (left)
/// [`Parser`]'s token output.
///
/// Parses input with the `left` [`Parser`], then feeds the output into the `right` [`Parser`].
/// Finally, it ignores the left [`Parser`]'s token and returns the right's.
pub(crate) fn right<'a, L, R, LT: 'a, RT: 'a>(left_p: L, right_p: R) -> ParserB<'a, RT>
where
    L: Parser<'a, LT> + 'a,
    R: Parser<'a, RT> + 'a,
{
    ParserB::new(move |pos: i32, input: &'a str| {
        left_p
            .parse(pos, input)
            .and_then(|(remainder, left_parsed)| {
                right_p
                    .parse(left_parsed.end_pos, remainder)
                    .map(|(remainder, right_parsed)| {
                        (
                            remainder,
                            Parsed {
                                start_pos: left_parsed.start_pos,
                                end_pos: right_parsed.end_pos,
                                token: right_parsed.token,
                            },
                        )
                    })
            })
    })
}

/// Creates a new [`Parser`] which chains together three parsers but ignores the first (left)
/// and last (right) [`Parser`]'s token output.
///
/// Parses input with the `left_p` [`Parser`], then feeds the output into the `right_p` [`Parser`].
/// Finally, it ignores the left and right [`Parser`]'s token and returns the middle's.
pub(crate) fn middle<'a, L, M, R, LT: 'a, MT: 'a, RT: 'a>(
    left_p: L,
    middle_p: M,
    right_p: R,
) -> ParserB<'a, MT>
where
    L: Parser<'a, LT> + 'a,
    M: Parser<'a, MT> + 'a,
    R: Parser<'a, RT> + 'a,
    MT: 'a,
{
    right(left_p, left(middle_p, right_p))
}

/// Creates a new [`Parser`] which runs the provided `parser` until it fails, returning
/// the result as a [`Vec<T>`].
///
/// If no tokens are matched, this parser will return an empty list.
/// If a parser fails with an error other than [`ParseError::NotMatched`],
/// this parser will fail and return that error.
pub(crate) fn many<'a, P, T>(parser: P) -> ParserB<'a, Vec<T>>
where
    P: Parser<'a, T> + 'a,
{
    ParserB::new(move |pos: i32, input: &'a str| {
        let mut tokens: Vec<T> = Vec::new();
        let mut end_pos = pos;
        let mut remainder = input;

        loop {
            match parser.parse(end_pos, remainder) {
                Ok((new_remainder, parsed)) => {
                    remainder = new_remainder;
                    end_pos = parsed.end_pos;
                    tokens.push(parsed.token);
                }

                Err(ParseError::NotMatched) => {
                    break;
                }

                Err(e) => {
                    // The parser found broken/malformed input, return error.
                    return Err(e);
                }
            }
        }

        Ok((
            remainder,
            Parsed {
                start_pos: pos,
                end_pos,
                token: tokens,
            },
        ))
    })
}

/// Creates a new [`Parser`] which runs the provided `parser` until it fails, returning
/// the result as a [`Vec<T>`]. Must match at least one token.
///
/// If no tokens are matched, this parser will return a [`ParseError:NotMatched`] error.
/// If a parser fails with an error other than [`ParseError::NotMatched`],
/// this parser will fail and return that error.
pub(crate) fn one_to_many<'a, P, T>(parser: P) -> ParserB<'a, Vec<T>>
where
    P: Parser<'a, T> + 'a,
    T: 'a,
{
    let p = many(parser);

    ParserB::new(move |pos: i32, input: &'a str| match p.parse(pos, input) {
        Ok((remainder, parsed)) => {
            // Return NotMatched if no tokens were matched.
            if parsed.token.is_empty() {
                Err(ParseError::NotMatched)
            } else {
                Ok((remainder, parsed))
            }
        }

        res => res,
    })
}

/// Creates a [`ParserB<'a, char>`] which parses the given char, returning it
/// as a token.
pub(crate) fn char_parser<'a>(value: char) -> ParserB<'a, char> {
    ParserB::new(move |pos: i32, input: &'a str| {
        if input.starts_with(value) {
            let s = String::from(value);

            Ok((
                &input[s.len()..],
                Parsed {
                    token: value,
                    start_pos: pos,
                    end_pos: pos + 1,
                },
            ))
        } else {
            Err(ParseError::NotMatched)
        }
    })
}

/// Creates a [`ParserB<'a, String>`] which parses the given string, returning it
/// as a token.
pub(crate) fn string_parser<'a>(value: impl Into<String>) -> ParserB<'a, String> {
    let str_ = value.into();
    if str_.is_empty() {
        unimplemented!("should return parser that always errors")
    }

    let mut chars = str_.chars();
    let first = fmap(char_parser(chars.next().unwrap()), String::from);

    chars.fold(first, |acc, p| chain(acc, char_parser(p)))
}

/// Creates a [`ParserB<'a, String>`] which takes characters until the `terminator` char is
/// reached.
pub(crate) fn take_until_char_parser<'a>(terminator: char) -> ParserB<'a, String> {
    ParserB::new(move |pos: i32, input: &'a str| {
        let token = input
            .to_string()
            .chars()
            .take_while(|x| *x != terminator)
            .collect::<String>();

        if token.is_empty() {
            Err(ParseError::NotMatched)
        } else {
            Ok((
                &input[token.len()..],
                Parsed {
                    start_pos: pos,
                    end_pos: pos + (token.chars().count() as i32),
                    token,
                },
            ))
        }
    })
}

/// Creates a [`ParserB<'a, String>`] which takes characters until it reaches one that is not
/// in the `allowed_chars` array.
pub(crate) fn take_while_chars_parser<'a>(allowed_chars: Vec<char>) -> ParserB<'a, String> {
    ParserB::new(move |pos: i32, input: &'a str| {
        let token = input
            .to_string()
            .chars()
            .take_while(|x| allowed_chars.contains(x))
            .collect::<String>();

        if token.is_empty() {
            Err(ParseError::NotMatched)
        } else {
            Ok((
                &input[token.len()..],
                Parsed {
                    start_pos: pos,
                    end_pos: pos + (token.chars().count() as i32),
                    token,
                },
            ))
        }
    })
}

/// Creates a [`Parser`] which parses lists of `item`s, separated by `delimiter`s.
///
/// If no items are found, this [`Parser`] will return an empty [`Vec<T>`].
/// Requires [`Fn() -> ParserB<'a, T>`] generators as they are used multiple times.
pub(crate) fn delimited_list<'a, P, T: 'a, D, DT: 'a>(item: P, delimiter: D) -> ParserB<'a, Vec<T>>
where
    P: Fn() -> ParserB<'a, T> + 'a,
    D: Fn() -> ParserB<'a, DT> + 'a,
    T: Clone,
{
    fmap(
        optional(chain(
            item(),
            optional(many(fmap_chain(delimiter(), item(), |_, i| i))),
        )),
        |t| t.unwrap_or_else(Vec::new),
    )
}

/// Creates a [`Parser`] which parses key value pairs in the following format:
/// <`key`><`delimiter`><`value`>. It returns a tuple of (`KT`, `VT`).
pub(crate) fn key_sep_value<'a, K, KT: 'a, D, DT: 'a, V, VT: 'a>(
    key: K,
    delimiter: D,
    value: V,
) -> ParserB<'a, (KT, VT)>
where
    K: Parser<'a, KT> + 'a,
    D: Parser<'a, DT> + 'a,
    V: Parser<'a, VT> + 'a,
{
    fmap_chain(key, right(delimiter, value), |k, v| (k, v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_parser() {
        let p = char_parser('c');
        let (remainder, parsed) = p
            .parse(0, "cd")
            .expect("Char parser `c` should successfully parse input `cd`");

        assert_eq!(
            parsed.token, 'c',
            "Char parser `c` should successfully parse input `cd`"
        );

        assert_eq!(
            remainder, "d",
            "Char parser `c` should produce remainder `d` for input `cd`"
        );

        let p = char_parser('c');
        let err = p
            .parse(0, "dc")
            .expect_err("Char parser `c` should fail on input `dc`");

        assert_eq!(
            err,
            ParseError::NotMatched,
            "Char parser `c` should return error `NotMatched` for input `dc`"
        );
    }

    #[test]
    fn test_string_parser() {
        let p = string_parser("Hello");
        let (remainder, parsed) = p
            .parse(0, "Hello world")
            .expect("String parser `Hello` should successfully parse input `Hello world`");

        assert_eq!(
            parsed.token, "Hello",
            "String parser `Hello` should successfully parse input `Hello world`"
        );

        assert_eq!(
            remainder, " world",
            "String parser `Hello` should produce remainder ` world` for input `Hello world`"
        );
    }

    #[test]
    fn test_string_literal_parser() {
        let p = middle(
            char_parser('"'),
            take_until_char_parser('"'),
            char_parser('"'),
        );

        let (remainder, parsed) = p.parse(0, "\"Hello! @#$123456789\"").expect(
            "String literal parser should successfully parse input `\"Hello! @#$123456789\"`",
        );

        assert_eq!(
            parsed.token,
            "Hello! @#$123456789",
            "String literal parser failed to parse `\"Hello! @#$123456789\"`.\n\tExpected: `Hello! @#$123456789`\n\tGot: {}",
            parsed.token
        );

        assert_eq!(
            remainder,
            "",
            "String literal parser produced incorrect remainder for input `\"Hello! @#$123456789\"`.\n\tExpected: ``\n\tGot: `{}`",
            remainder
        );
    }

    #[test]
    fn test_variable_name_parser() {
        let allowed_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789-_"
            .chars()
            .collect::<Vec<char>>();

        let p = take_while_chars_parser(allowed_chars.clone());

        let (remainder, parsed) = p
            .parse(0, "helloWorld: ")
            .expect("Variable name parser should successfully parse input `helloWorld: `");

        assert_eq!(
            parsed.token,
            "helloWorld",
            "Variable name parser failed to parse `helloWorld: `.\n\tExpected: `helloWorld`\n\tGot: {}",
            parsed.token
        );

        assert_eq!(
            remainder,
            ": ",
            "Variable name parser produced incorrect remainder for input `helloWorld: `.\n\tExpected: `: `\n\tGot: `{}`",
            remainder
        );
    }

    #[test]
    fn test_array_parser() {
        let valid_input = r#"["h", "hello", "worlddd"]"#;
        let valid_output = vec!["h", "hello", "worlddd"];

        let invalid_input = r#"["hello" "world",, "goodbye", "aaaa"]"#;

        let string_literal_p = || {
            middle(
                char_parser('"'),
                take_until_char_parser('"'),
                char_parser('"'),
            )
        };

        let ws_chars = vec![' ', '\t'];
        let ws = || optional(take_while_chars_parser(ws_chars.clone()));

        let str_element_p = || middle(ws(), string_literal_p(), ws());

        let delimiter = || middle(ws(), char_parser(','), ws());

        let p = middle(
            fmap_chain(char_parser('['), ws(), |_, _| ()),
            delimited_list(str_element_p, delimiter),
            fmap_chain(ws(), char_parser(']'), |_, _| ()),
        );

        let (remainder, parsed) = p.parse(0, valid_input).expect(&format!(
            "Array parser should successfully parse input `{}`",
            valid_input
        ));

        assert_eq!(
            parsed.token, valid_output,
            "Array parser failed to parse `{}`.\n\tExpected: `{:?}`\n\tGot: `{:?}`",
            valid_input, valid_output, parsed.token,
        );

        assert_eq!(
            remainder, "",
            "Array parser produced incorrect remainder for input `{}`.\n\tExpected: `{:?}`\n\tGot: `{:?}`",
            valid_input, "", remainder,
        );

        p.parse(0, invalid_input).expect_err(&format!(
            "Array parser should not successfully parse input `{}`",
            invalid_input
        ));
    }

    #[test]
    fn test_key_value() {
        let allowed_variable_chars =
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789-_"
                .chars()
                .collect::<Vec<char>>();

        let variable_name_p = || take_while_chars_parser(allowed_variable_chars.clone());

        let ws_chars = vec![' ', '\t'];
        let ws = || optional(take_while_chars_parser(ws_chars.clone()));

        let string_literal_p = || {
            middle(
                char_parser('"'),
                take_until_char_parser('"'),
                char_parser('"'),
            )
        };

        let str_element_p = || middle(ws(), string_literal_p(), ws());

        let delimiter_p = || middle(ws(), char_parser(':'), ws());

        let p = key_sep_value(variable_name_p(), delimiter_p(), str_element_p());

        let valid_input = r#"helloWorld: "value""#;
        let valid_output = ("helloWorld".to_string(), "value".to_string());
        let invalid_input = r#"h'elloWorld: "value""#;

        let (remainder, parsed) = p.parse(0, valid_input).expect(&format!(
            "Key-value parser should successfully parse input `{}`",
            valid_input
        ));

        assert_eq!(
            parsed.token, valid_output,
            "Key-value parser failed to parse `{}`.\n\tExpected: `{:?}`\n\tGot: `{:?}`",
            valid_input, valid_output, parsed.token,
        );

        assert_eq!(
            remainder, "",
            "Key-value parser produced incorrect remainder for input `{}`.\n\tExpected: `{:?}`\n\tGot: `{:?}`",
            valid_input, "", remainder,
        );

        p.parse(0, invalid_input).expect_err(&format!(
            "Key-value parser should not successfully parse input `{}`",
            invalid_input
        ));
    }
}
