use std::collections::HashMap;

use crate::balsa_types::{BalsaExpression, BalsaValue};
use crate::converters::tuple_vec_to_map;
use crate::errors::{BalsaCompileError, BalsaError, TemplateErrorContext, TemplateParseFail};
use crate::parser::{
    char_parser, delimited_list, fmap, fmap_chain, fmap_result, key_sep_value, many, middle,
    optional, or, right, string_parser, take_until_char_parser, take_while_chars_parser,
    ParseError, Parser, ParserB,
};
use crate::BalsaType;

/// Exposes methods for parsing Balsa templates.
pub(crate) struct BalsaParser;

impl BalsaParser {
    /// Parses a string input to a list of [`BalsaToken`]s.
    pub(crate) fn parse(input: String) -> Result<Vec<BalsaToken>, BalsaError> {
        let p = balsa_p();

        p.parse(0, &input).map(|(_, t)| t.token).map_err(|_| {
            BalsaError::CompileError(BalsaCompileError::TemplateParseFail(TemplateErrorContext {
                pos: 0, // TODO
                error: TemplateParseFail::Generic,
            }))
        })
    }
}

/// Represents a key-value set from a block.
///
/// i.e. `defaultValue: "test", type: string`
pub(crate) type OptionsMap = HashMap<String, BalsaExpression>;

/// Contains contextual information about a block.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(crate) struct Block<T> {
    pub(crate) start_pos: i32,
    pub(crate) end_pos: i32,
    pub(crate) token: T,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Declaration {
    pub(crate) identifier: BalsaExpression,
    pub(crate) variable_type: BalsaExpression,
    pub(crate) value: BalsaExpression,
}

/// Intermediate representation for a parameter block.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParameterBlockIntermediate {
    /// The name of the variable being referenced.
    pub(crate) variable_name: BalsaExpression,
    /// The type of the variable expected.
    pub(crate) variable_type: BalsaExpression,
    /// A list of optional options.
    pub(crate) options: Option<OptionsMap>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BalsaToken {
    DeclarationBlock(Block<Vec<Declaration>>),
    ParameterBlock(Block<ParameterBlockIntermediate>),
}

const STR_LITERAL_QUOTE: char = '"';
const ALLOWED_VARIABLE_CHARACTERS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789-_";
const DIGITS: &str = "1234567890";
const KEY_VALUE_DELIMETER: char = ':';
const LIST_ELEMENT_DELIMETER: char = ',';
const DECLARATION_DELIMITER: char = '=';

fn parameter_open_bracket_p<'a>() -> ParserB<'a, ()> {
    fmap(string_parser("{{"), |_, _| ())
}

fn declaration_open_bracket_p<'a>() -> ParserB<'a, ()> {
    fmap(string_parser("{{@"), |_, _| ())
}

fn closing_bracket_p<'a>() -> ParserB<'a, ()> {
    fmap(string_parser("}}"), |_, _| ())
}

fn ws_p<'a>() -> ParserB<'a, ()> {
    let ws_chars = vec![' ', '\t', '\n'];

    fmap(optional(take_while_chars_parser(ws_chars)), |_, _| ())
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

fn variable_with_type_p<'a>() -> ParserB<'a, (BalsaExpression, BalsaExpression)> {
    key_sep_value(balsa_expr_p(), key_value_delimiter_p(), balsa_expr_p())
}

fn string_literal_p<'a>() -> ParserB<'a, BalsaValue> {
    fmap(
        middle(
            char_parser('"'),
            take_until_char_parser('"'),
            char_parser('"'),
        ),
        |s, _| BalsaValue::String(s),
    )
}

fn int_literal_p<'a>() -> ParserB<'a, BalsaValue> {
    let digits = DIGITS.chars().collect::<Vec<char>>();
    let digit_p = take_while_chars_parser(digits);

    fmap_result(digit_p, |token, _| match token.parse::<i64>() {
        Ok(val) => Ok(BalsaValue::Integer(val)),
        Err(_) => Err(ParseError::MalformedInput(0)),
    })
}

fn balsa_type_p<'a>() -> ParserB<'a, BalsaType> {
    // TODO: or macro or similar shortcut for scalability
    or(
        fmap(string_parser("string"), |_, _| BalsaType::String),
        or(
            fmap(string_parser("color"), |_, _| BalsaType::Color),
            or(
                fmap(string_parser("int"), |_, _| BalsaType::Integer),
                fmap(string_parser("float"), |_, _| BalsaType::Float),
            ),
        ),
    )
}

fn balsa_value_p<'a>() -> ParserB<'a, BalsaValue> {
    or(string_literal_p(), int_literal_p())
}

fn balsa_expr_p<'a>() -> ParserB<'a, BalsaExpression> {
    or(
        fmap(balsa_value_p(), |v, _| BalsaExpression::Value(v)),
        or(
            fmap(balsa_type_p(), |t, _| BalsaExpression::Type(t)),
            fmap(variable_name_p(), |v, _| BalsaExpression::Identifier(v)),
        ),
    )
}

fn key_value_delimiter_p<'a>() -> ParserB<'a, ()> {
    fmap(ws_padded_p(char_parser(KEY_VALUE_DELIMETER)), |_, _| ())
}

fn key_value_p<'a>() -> ParserB<'a, (String, BalsaExpression)> {
    key_sep_value(variable_name_p(), key_value_delimiter_p(), balsa_expr_p())
}

fn list_delimeter<'a>() -> ParserB<'a, ()> {
    fmap(ws_padded_p(char_parser(LIST_ELEMENT_DELIMETER)), |_, _| ())
}

fn declaration_delimiter_p<'a>() -> ParserB<'a, ()> {
    fmap(ws_padded_p(char_parser(DECLARATION_DELIMITER)), |_, _| ())
}

fn declaration_p<'a>() -> ParserB<'a, Declaration> {
    fmap_chain(
        variable_with_type_p(),
        right(declaration_delimiter_p(), balsa_expr_p()),
        |((identifier, variable_type), _), (value, _)| Declaration {
            identifier,
            variable_type,
            value,
        },
    )
}

fn declaration_block_p<'a>() -> ParserB<'a, BalsaToken> {
    fmap(
        middle(
            declaration_open_bracket_p(),
            ws_padded_p(delimited_list(declaration_p, list_delimeter)),
            closing_bracket_p(),
        ),
        |d, ctx| {
            BalsaToken::DeclarationBlock(Block {
                start_pos: ctx.start_pos,
                end_pos: ctx.end_pos,
                token: d,
            })
        },
    )
}

fn parameter_block_p<'a>() -> ParserB<'a, BalsaToken> {
    fmap(
        middle(
            parameter_open_bracket_p(),
            ws_padded_p(fmap_chain(
                variable_with_type_p(),
                optional(right(
                    list_delimeter(),
                    delimited_list(key_value_p, list_delimeter),
                )),
                |((variable_name, variable_type), _), (options_list, _)| {
                    let options = options_list.map(tuple_vec_to_map);

                    ParameterBlockIntermediate {
                        variable_name,
                        variable_type,
                        options,
                    }
                },
            )),
            closing_bracket_p(),
        ),
        |p, ctx| {
            BalsaToken::ParameterBlock(Block {
                start_pos: ctx.start_pos,
                end_pos: ctx.end_pos,
                token: p,
            })
        },
    )
}

/// Parses any kind of block into a BalsaToken.
fn block_p<'a>() -> ParserB<'a, BalsaToken> {
    or(parameter_block_p(), declaration_block_p())
}

fn balsa_p<'a>() -> ParserB<'a, Vec<BalsaToken>> {
    fmap(
        many(right(
            take_until_char_parser('{'),
            or(
                fmap(block_p(), |v, _| Some(v)),
                fmap(take_while_chars_parser(vec!['{']), |_, _| None),
            ),
        )),
        |v, _| v.into_iter().flatten().collect(),
    )
}

#[cfg(test)]
mod tests {
    use crate::BalsaType;

    use super::*;

    #[test]
    fn test_parameter_block_p() {
        let valid_input = r#"{{ helloWorld: color, defaultValue: "hello world" }}"#;
        let mut valid_options = HashMap::new();
        valid_options.insert(
            "defaultValue".to_string(),
            BalsaExpression::Value(BalsaValue::String("hello world".to_string())),
        );
        let valid_output = BalsaToken::ParameterBlock(Block {
            start_pos: 0,
            end_pos: 52,
            token: ParameterBlockIntermediate {
                variable_name: BalsaExpression::Identifier("helloWorld".to_string()),
                variable_type: BalsaExpression::Type(BalsaType::Color),
                options: Some(valid_options),
            },
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

    #[test]
    fn test_declaration_block_p() {
        let valid_input = r#"{{@ hello: string     = "world" }}"#;
        let mut valid_declarations = Vec::new();
        valid_declarations.push(Declaration {
            identifier: BalsaExpression::Identifier("hello".to_string()),
            variable_type: BalsaExpression::Type(BalsaType::String),
            value: BalsaExpression::Value(BalsaValue::String("world".to_string())),
        });
        let valid_output = BalsaToken::DeclarationBlock(Block {
            start_pos: 0,
            end_pos: valid_input.len() as i32,
            token: valid_declarations,
        });
        let p = declaration_block_p();

        let (_, parsed) = p.parse(0, valid_input).expect(&format!(
            "Declaration block parser should successfully parse input `{}`",
            valid_input
        ));

        assert!(
            PartialEq::eq(&parsed.token, &valid_output),
            "Declaration block parser failed to parse `{}`.\n\tExpected: `{:?}`\n\tGot: `{:?}`",
            valid_input,
            valid_output,
            parsed.token
        );
    }

    #[test]
    fn test_balsa_p() {
        let valid_input = r#"
        <html>
            <head>
                {{@
                    test: string = "hello"
                }}
            </head>
            <body>
                <span>{{ helloWorld: string, defaultValue: "test" }}</span>
            </body>
        </html>
        "#;

        let mut valid_declarations = Vec::new();
        valid_declarations.push(Declaration {
            identifier: BalsaExpression::Identifier("test".to_string()),
            variable_type: BalsaExpression::Type(BalsaType::String),
            value: BalsaExpression::Value(BalsaValue::String("hello".to_string())),
        });

        let valid_declaration_output = BalsaToken::DeclarationBlock(Block {
            start_pos: 51,
            end_pos: 116,
            token: valid_declarations,
        });

        let mut valid_parameter_options = HashMap::new();
        valid_parameter_options.insert(
            "defaultValue".to_string(),
            BalsaExpression::Value(BalsaValue::String("test".to_string())),
        );

        let valid_parameter_output = BalsaToken::ParameterBlock(Block {
            start_pos: 178,
            end_pos: 224,
            token: ParameterBlockIntermediate {
                variable_name: BalsaExpression::Identifier("helloWorld".to_string()),
                variable_type: BalsaExpression::Type(BalsaType::String),
                options: Some(valid_parameter_options),
            },
        });

        let valid_output = vec![valid_declaration_output, valid_parameter_output];

        let p = balsa_p();

        let (_, parsed) = p.parse(0, valid_input).expect(&format!(
            "Balsa parser should successfully parse input `{}`",
            valid_input
        ));

        assert!(
            PartialEq::eq(&parsed.token, &valid_output),
            "Balsa parser failed to parse `{}`.\n\tExpected: `{:?}`\n\tGot: `{:?}`",
            valid_input,
            valid_output,
            parsed.token
        );
    }
}
