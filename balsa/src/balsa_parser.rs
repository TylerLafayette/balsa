use std::collections::HashMap;

use crate::balsa_types::{BalsaExpression, BalsaValue};
use crate::converters::tuple_vec_to_map;
use crate::parser::{
    char_parser, delimited_list, fmap, fmap_chain, key_sep_value, middle, optional, or, right,
    string_parser, take_until_char_parser, take_while_chars_parser, ParseError, Parsed, Parser,
    ParserB,
};

/// Represents a key-value set from a block.
///
/// i.e. `defaultValue: "test", type: string`
type OptionsMap = HashMap<String, BalsaExpression>;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Declaration {
    name: String,
    value: BalsaExpression,
}

/// Intermediate representation for a parameter block.
#[derive(Debug, Clone, PartialEq)]
struct ParameterBlockIntermediate {
    /// The name of the variable being referenced.
    variable_name: BalsaExpression,
    options: Option<OptionsMap>,
}

#[derive(Debug, Clone, PartialEq)]
enum BalsaToken {
    DeclarationBlock(Vec<Declaration>),
    ParameterBlock(ParameterBlockIntermediate),
}

const STR_LITERAL_QUOTE: char = '"';
const ALLOWED_VARIABLE_CHARACTERS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789-_";
const DIGITS: &str = "1234567890";
const KEY_VALUE_DELIMETER: char = ':';
const LIST_ELEMENT_DELIMETER: char = ',';

fn parameter_open_bracket_p<'a>() -> ParserB<'a, ()> {
    fmap(string_parser("{{"), |_| ())
}

fn declaration_open_bracket_p<'a>() -> ParserB<'a, ()> {
    fmap(string_parser("{{"), |_| ())
}

fn closing_bracket_p<'a>() -> ParserB<'a, ()> {
    fmap(string_parser("}}"), |_| ())
}

fn ws_p<'a>() -> ParserB<'a, ()> {
    let ws_chars = vec![' ', '\t', '\n'];

    fmap(optional(take_while_chars_parser(ws_chars)), |_| ())
}

fn ws_padded_p<'a, P, T: 'a>(parser: P) -> ParserB<'a, T>
where
    P: Parser<'a, T> + 'a,
{
    middle(ws_p(), parser, ws_p())
}

fn variable_name_p<'a>() -> ParserB<'a, String> {
    let allowed_chars = ALLOWED_VARIABLE_CHARACTERS.chars().collect::<Vec<char>>();

    take_while_chars_parser(allowed_chars)
}

fn string_literal_p<'a>() -> ParserB<'a, BalsaValue> {
    fmap(
        middle(
            char_parser('"'),
            take_until_char_parser('"'),
            char_parser('"'),
        ),
        BalsaValue::String,
    )
}

fn int_literal_p<'a>() -> ParserB<'a, BalsaValue> {
    let digits = DIGITS.chars().collect::<Vec<char>>();
    let digit_p = take_while_chars_parser(digits);

    // TODO: fmap_failable combinator in parser.rs
    ParserB::new(move |pos: i32, input: &'a str| {
        digit_p.parse(pos, input).and_then(|(remainder, parsed)| {
            match parsed.token.parse::<i64>() {
                Ok(val) => Ok((
                    remainder,
                    Parsed {
                        start_pos: pos,
                        end_pos: parsed.end_pos,
                        token: BalsaValue::Integer(val),
                    },
                )),

                Err(_) => Err(ParseError::MalformedInput(pos)),
            }
        })
    })
}

fn balsa_value_p<'a>() -> ParserB<'a, BalsaValue> {
    or(string_literal_p(), int_literal_p())
}

fn balsa_expr_p<'a>() -> ParserB<'a, BalsaExpression> {
    fmap(balsa_value_p(), BalsaExpression::Value)
}

fn key_value_delimiter_p<'a>() -> ParserB<'a, ()> {
    fmap(ws_padded_p(char_parser(KEY_VALUE_DELIMETER)), |_| ())
}

fn key_value_p<'a>() -> ParserB<'a, (String, BalsaExpression)> {
    key_sep_value(variable_name_p(), key_value_delimiter_p(), balsa_expr_p())
}

fn list_delimeter<'a>() -> ParserB<'a, ()> {
    fmap(ws_padded_p(char_parser(LIST_ELEMENT_DELIMETER)), |_| ())
}

fn parameter_block_p<'a>() -> ParserB<'a, BalsaToken> {
    middle(
        parameter_open_bracket_p(),
        ws_padded_p(fmap_chain(
            variable_name_p(),
            optional(right(
                list_delimeter(),
                delimited_list(key_value_p, list_delimeter),
            )),
            |variable_name, options_list| {
                let options = options_list.map(tuple_vec_to_map);

                BalsaToken::ParameterBlock(ParameterBlockIntermediate {
                    variable_name: BalsaExpression::Variable(variable_name),
                    options,
                })
            },
        )),
        closing_bracket_p(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_block_p() {
        let valid_input = r#"{{ helloWorld, defaultValue: "hello world" }}"#;
        let mut valid_options = HashMap::new();
        valid_options.insert(
            "defaultValue".to_string(),
            BalsaExpression::Value(BalsaValue::String("hello world".to_string())),
        );
        let valid_output = BalsaToken::ParameterBlock(ParameterBlockIntermediate {
            variable_name: BalsaExpression::Variable("helloWorld".to_string()),
            options: Some(valid_options),
        });

        let p = parameter_block_p();

        let (_, parsed) = p.parse(0, valid_input).expect(&format!(
            "Parameter block parser should successfully parse input `{}`",
            valid_input
        ));

        assert!(
            PartialEq::eq(&parsed.token, &valid_output),
            "Parameter block parser failed to parse `{}`.\n\tExpected: `{:?}`\n\tGot: `{:?}`",
            valid_input,
            valid_output,
            parsed.token
        );
    }
}
