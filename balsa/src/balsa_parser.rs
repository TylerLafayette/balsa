use crate::parser::{char_parser, fmap, string_parser, ParserB};
use crate::types::BalsaType;

struct BalsaDeclaration {
    name: String,
    value: BalsaType,
}

struct ParameterBlock {
    variable_name: String,
    default_value: Option<BalsaType>,
}

enum BalsaToken {
    DeclarationBlock(Vec<BalsaDeclaration>),
    ParameterBlock(ParameterBlock),
}

fn open_bracket_parser<'a>() -> ParserB<'a, ()> {
    fmap(string_parser("{{"), |_| ())
}

fn closing_bracket_parser<'a>() -> ParserB<'a, ()> {
    fmap(string_parser("}}"), |_| ())
}
