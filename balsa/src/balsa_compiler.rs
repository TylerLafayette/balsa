use std::collections::HashMap;

use crate::{
    balsa_parser::{BalsaToken, Block, Declaration, ParameterBlockIntermediate},
    errors::{BalsaCompileError, BalsaError, TemplateErrorContext},
    parameter_names, BalsaResult, BalsaType, BalsaValue,
};

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct CompiledTemplate {
    global_scope: Scope,
    replacements: Vec<ReplacementInstruction>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct Scope {
    variables: HashMap<String, BalsaValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ReplacementInstruction {
    start_pos: usize,
    end_pos: usize,
    replace_with: ReplaceWith,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ReplaceWith {
    Parameter(ParameterDescription),
    Nothing,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParameterDescription {
    variable_name: String,
    variable_type: BalsaType,
    default_value: Option<BalsaValue>,
}

/// Struct which provides compiler methods.
pub(crate) struct Compiler {
    global_scope: Scope,
    replacements: Vec<ReplacementInstruction>,
}

impl Compiler {
    /// Compiles a template from a list of tokens/AST from the parser.
    pub(crate) fn compile_from_tokens(tokens: &[BalsaToken]) -> BalsaResult<CompiledTemplate> {
        let mut compiler = Self {
            global_scope: Scope::default(),
            replacements: Vec::new(),
        };

        for token in tokens {
            match token {
                BalsaToken::ParameterBlock(p) => compiler.parse_param_block(p)?,
                BalsaToken::DeclarationBlock(d) => compiler.parse_dec_block(d)?,
            }
        }

        Ok(CompiledTemplate {
            global_scope: compiler.global_scope,
            replacements: compiler.replacements,
        })
    }

    fn parse_param_block(&mut self, block: &Block<ParameterBlockIntermediate>) -> BalsaResult<()> {
        let i = block.token.variable_name.as_identifier().ok_or_else(|| {
            BalsaError::invalid_identifier_in_parameter_block(
                block.start_pos as usize,
                block.token.variable_name.clone(),
            )
        })?;

        let type_ = block.token.variable_type.as_type().ok_or_else(|| {
            BalsaError::invalid_type_expression(
                block.start_pos as usize,
                block.token.variable_type.clone(),
            )
        })?;

        let mut param_description = ParameterDescription {
            variable_name: i,
            variable_type: type_,
            default_value: None,
        };

        if let Some(map) = &block.token.options {
            for (key, value) in map {
                match key.as_str() {
                    parameter_names::DEFAULT_VALUE => {
                        let default_value = value
                            .as_value()
                            .ok_or_else(|| {
                                BalsaError::invalid_expression(
                                    block.start_pos as usize,
                                    value.clone(),
                                )
                            })?
                            .try_cast(type_)
                            .map_err(|error| {
                                BalsaError::new_compile_error(BalsaCompileError::InvalidTypeCast(
                                    TemplateErrorContext {
                                        pos: block.start_pos as usize,
                                        error,
                                    },
                                ))
                            })?;

                        param_description.default_value = Some(default_value);
                    }
                    _ => {
                        return Err(BalsaError::invalid_parameter(
                            block.start_pos as usize,
                            key.clone(),
                        ))
                    }
                }
            }
        }

        let instr = ReplacementInstruction {
            start_pos: block.start_pos as usize,
            end_pos: block.end_pos as usize,
            replace_with: ReplaceWith::Parameter(param_description),
        };

        self.replacements.push(instr);

        Ok(())
    }

    fn parse_dec_block(&mut self, block: &Block<Vec<Declaration>>) -> BalsaResult<()> {
        for declaration in &block.token {
            let identifier = declaration.identifier.as_identifier().ok_or_else(|| {
                BalsaError::invalid_identifier_in_declaration_block(
                    block.start_pos as usize,
                    declaration.identifier.clone(),
                )
            })?;

            let type_ = declaration.variable_type.as_type().ok_or_else(|| {
                BalsaError::invalid_type_expression(
                    block.start_pos as usize,
                    declaration.variable_type.clone(),
                )
            })?;

            let value = declaration
                .value
                .as_value()
                .ok_or_else(|| {
                    BalsaError::invalid_expression(
                        block.start_pos as usize,
                        declaration.value.clone(),
                    )
                })?
                .try_cast(type_)
                .map_err(|error| {
                    BalsaError::new_compile_error(BalsaCompileError::InvalidTypeCast(
                        TemplateErrorContext {
                            pos: block.start_pos as usize,
                            error,
                        },
                    ))
                })?;

            self.global_scope.variables.insert(identifier, value);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::balsa_types::BalsaExpression;

    use super::*;

    /// Converts a [`HashMap`] to a DeclarationBlock.
    fn map_to_declaration_block(
        start_pos: i32,
        end_pos: i32,
        map: HashMap<String, (BalsaType, BalsaValue)>,
    ) -> BalsaToken {
        let ds = map
            .into_iter()
            .map(|(identifier, (variable_type, value))| Declaration {
                identifier: BalsaExpression::Identifier(identifier),
                variable_type: BalsaExpression::Type(variable_type),
                value: BalsaExpression::Value(value),
            })
            .collect::<Vec<Declaration>>();

        BalsaToken::DeclarationBlock(Block {
            start_pos,
            end_pos,
            token: ds,
        })
    }

    #[test]
    fn test_compiler() {
        let dec_block = map_to_declaration_block(
            0,
            30,
            HashMap::from([
                (
                    "helloWorld".to_string(),
                    (BalsaType::String, BalsaValue::String("goodbye".to_string())),
                ),
                (
                    "favoriteNumber".to_string(),
                    (BalsaType::Integer, BalsaValue::Integer(1)),
                ),
            ]),
        );

        let param_block = BalsaToken::ParameterBlock(Block {
            start_pos: 40,
            end_pos: 80,
            token: ParameterBlockIntermediate {
                variable_name: BalsaExpression::Identifier("testInt".to_string()),
                variable_type: BalsaExpression::Type(BalsaType::Integer),
                options: Some(HashMap::from([(
                    "defaultValue".to_string(),
                    BalsaExpression::Value(BalsaValue::Integer(1)),
                )])),
            },
        });

        let tokens = vec![dec_block, param_block];

        let output =
            Compiler::compile_from_tokens(&tokens).expect("failed to compile from token list");

        let values = [
            (
                "helloWorld",
                Some(BalsaValue::String("goodbye".to_string())),
            ),
            ("favoriteNumber", Some(BalsaValue::Integer(1))),
        ];

        for (id, val) in values {
            let item = output
                .global_scope
                .variables
                .get("helloWorld")
                .map(|x| x.clone());

            assert_eq!(
                item,
                Some(BalsaValue::String("goodbye".to_string())),
                "Global scope variable `{}` set incorrectly.\n\tExpected: `{:?}`\n\tGot: `{:?}`",
                id,
                val,
                item
            );
        }

        let params = vec![ReplacementInstruction {
            start_pos: 40,
            end_pos: 80,
            replace_with: ReplaceWith::Parameter(ParameterDescription {
                variable_name: "testInt".to_string(),
                variable_type: BalsaType::Integer,
                default_value: Some(BalsaValue::Integer(1)),
            }),
        }];

        assert_eq!(
            output.replacements, params,
            "Global scope replacements not generated correctly.\n\tExpected: `{:?}`\n\tGot: `{:?}`",
            params, output.replacements
        );
    }
}
