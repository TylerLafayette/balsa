use std::str::Chars;

use crate::{
    balsa_compiler::{CompiledTemplate, ReplaceWith, ReplacementInstruction},
    errors::BalsaError,
    BalsaParameters, BalsaResult, BalsaValue,
};

/// Provides methods for rendering a compiled template.
///
/// Renderers are meant to be used a single timk
pub(crate) struct Renderer<'a> {
    raw_template: &'a str,
    compiled_template: &'a CompiledTemplate,
}

/// Holds state for a currently rendering template.
struct RenderContext<'a> {
    output: String,
    chars_written: usize,
    chars: Chars<'a>,
    parameters: &'a BalsaParameters,
}

impl<'a> Renderer<'a> {
    /// Creates a new [`Renderer`] for the given template.
    pub(crate) fn new(raw_template: &'a str, compiled_template: &'a CompiledTemplate) -> Self {
        let p = BalsaParameters::default();

        Self {
            raw_template,
            compiled_template,
        }
    }

    /// Renders the template with the given [`BalsaParameters`].
    pub(crate) fn render_with_parameters(
        &mut self,
        parameters: &'a BalsaParameters,
    ) -> BalsaResult<String> {
        let mut ctx = RenderContext::new(self.raw_template, parameters);

        for replacement in &self.compiled_template.replacements {
            ctx.next(replacement);
        }

        Ok(ctx.output())
    }
}

impl<'a> RenderContext<'a> {
    /// Creates a new [`RenderContext`] from the supplied raw template source.
    fn new(raw_template: &'a str, parameters: &'a BalsaParameters) -> Self {
        Self {
            output: String::new(),
            chars_written: 0,
            chars: raw_template.chars(),
            parameters,
        }
    }

    /// Processes the next ReplacementInstruction.
    fn next(&mut self, replacement: &ReplacementInstruction) -> BalsaResult<()> {
        self.prepend_missing_chars(replacement);

        match &replacement.replace_with {
            ReplaceWith::Parameter(p) => {
                let value = self
                    .parameters
                    .get(&p.variable_name)
                    .or_else(|| p.default_value.clone());

                match value {
                    None => return Err(BalsaError::missing_parameter(p.variable_name.clone())),
                    Some(v) => {
                        let v = v.try_cast(p.variable_type).map_err(|e| {
                            BalsaError::invalid_parameter_type(
                                p.variable_name.clone(),
                                v.clone(),
                                v.get_type(),
                                p.variable_type,
                            )
                        })?;

                        match &v {
                            BalsaValue::String(s) => self.output.push_str(s),
                            BalsaValue::Color(s) => self.output.push_str(s),
                            BalsaValue::Integer(i) => self.output.push_str(&i.to_string()),
                            BalsaValue::Float(f) => self.output.push_str(&f.to_string()),
                        }
                    }
                }
            }
            ReplaceWith::Nothing => {}
        }

        Ok(())
    }

    /// Prepends chars that come before a replacement block that haven't previously been prepended
    /// and drops chars up to the replacement's `end_pos`.
    fn prepend_missing_chars(&mut self, replacement: &ReplacementInstruction) {
        if self.chars_written < replacement.start_pos {
            let n = replacement.start_pos - self.chars_written;
            self.output
                .push_str(&(&mut self.chars).take(n).collect::<String>());

            self.chars_written += n;
        }

        if self.chars_written < replacement.end_pos {
            let n = replacement.end_pos - self.chars_written;
            (&mut self.chars).skip(n).for_each(drop);
        }
    }

    /// Flushes the char buffer and returns the output of the render, consuming `self`.
    fn output(mut self) -> String {
        // Flush remaining chars.
        self.output.push_str(&(&mut self.chars).collect::<String>());

        self.output
    }
}
