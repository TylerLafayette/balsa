/// Represents a parsed token.
#[derive(Debug, PartialEq)]
pub(crate) struct Parsed<T> {
    start_pos: i32,
    end_pos: i32,
    token: T,
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

pub(crate) struct ParserB<'a, T> {
    parser: Box<dyn Parser<'a, T>>,
}

impl<'a, T> ParserB<'a, T> {
    fn new<P>(parser: P) -> ParserB<'a, T>
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

// /// This trait provides the `fmap` function which lets you turn
// /// a functor of one type into a functor of another.
// trait Functor<A, B, C> {
//     fn fmap<F>(&self, function: F) -> C
//     where
//         F: Fn(A) -> B;
// }
//
// /// This trait provides some methods used for an applicative
// /// functor.
// trait Applicative {
//     fn or(&self, alternative: Self) -> Self;
//     fn and(&self, combine_with: Self) -> Self;
//     fn left(&self, right: Self) -> Self;
//     fn right(&self, right: Self) -> Self;
// }

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
    /// Combines two chars into a String.
    fn combine(&self, with: char) -> String {
        let mut s = String::new();
        s.push_str(&self);
        s.push(with);
        s
    }
}

impl Combinable<String, String> for String {
    fn combine(&self, with: String) -> String {
        format!("{}{}", self, with)
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

/// Maps a [`Parser<'a, T>`] to a [`Parser<'a, O>`] using the provided
/// function `F`.
pub(crate) fn fmap<'a, P, T, O, F>(parser: P, function: F) -> ParserB<'a, O>
where
    P: Parser<'a, T> + 'a,
    F: Fn(T) -> O + 'a,
{
    ParserB::new(move |pos: i32, input: &'a str| {
        parser.parse(pos, input).map(|(remainder, output)| {
            (
                remainder,
                Parsed {
                    start_pos: output.start_pos,
                    end_pos: output.end_pos,
                    token: function(output.token),
                },
            )
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
    LT: Combinable<RT, O>,
{
    ParserB::new(move |pos: i32, input: &'a str| {
        left.parse(pos, input).and_then(|(remainder, left_parsed)| {
            right
                .parse(left_parsed.end_pos, remainder)
                .map(|(remainder, right_parsed)| (remainder, left_parsed.combine(right_parsed)))
        })
    })
}

/// Creates a [`ParserB`] which parses the given char, returning it
/// as a token.
pub(crate) fn char_parser<'a>(value: char) -> ParserB<'a, char> {
    ParserB::new(move |pos: i32, input: &'a str| {
        if input.chars().nth(0) == Some(value) {
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

/// Creates a [`ParserB`] which parses the given string, returning it
/// as a token.
pub(crate) fn string_parser<'a>(value: impl Into<String>) -> ParserB<'a, String> {
    let str_ = value.into();
    if str_.len() == 0 {
        unimplemented!("should return parser that always errors")
    }

    let mut chars = str_.chars();
    let first = fmap(char_parser(chars.next().unwrap()), String::from);

    chars.fold(first, |acc, p| chain(acc, char_parser(p)))
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
}
