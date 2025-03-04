use bumpalo::{collections::Vec, Bump};
use miette::{SourceOffset, SourceSpan};
use shin_versions::MessageCommandStyle;

use crate::{
    message_parser::{lint::diagnostics::LineReport, MessageCommand, MessageToken},
    reactor::AnyStringSource,
};

pub mod diagnostics {
    use miette::{Diagnostic, SourceSpan};
    use thiserror::Error;

    #[derive(Diagnostic, Debug, Error)]
    #[error("Unknown layout command")]
    pub struct UnknownCommand {
        #[label = "This command is not recognized"]
        pub err_span: SourceSpan,
        #[help]
        pub advice: Option<&'static str>,
    }

    #[derive(Diagnostic, Debug, Error)]
    #[error("Layout command needs an argument")]
    #[diagnostic(help("The argument has to be terminated with a dot (`.`) to be recognized"))]
    pub struct MissingCommandArgument {
        #[label]
        pub err_span: SourceSpan,
    }

    #[derive(Diagnostic, Debug, Error)]
    pub enum AnyDiagnostic {
        #[error(transparent)]
        #[diagnostic(transparent)]
        UnknownCommand(#[from] UnknownCommand),
        #[error(transparent)]
        #[diagnostic(transparent)]
        MissingCommandArgument(#[from] MissingCommandArgument),
    }

    #[derive(Diagnostic, Debug, Error)]
    #[diagnostic()]
    #[error("line #{index} contains errors")]
    pub struct LineReport {
        #[source_code]
        pub src: String,
        pub index: u32,
        #[related]
        pub diagnostics: Vec<AnyDiagnostic>,
    }
}

fn diagnose(
    sink: &mut std::vec::Vec<diagnostics::AnyDiagnostic>,
    token: MessageToken,
    span: SourceSpan,
) {
    let MessageToken::Command(command_token) = token else {
        return;
    };
    let Some(command) = MessageCommand::parse(command_token.command) else {
        let advice = if command_token.command == '@' {
            Some("did you forget to pass --message-style modernize?")
        } else {
            None
        };

        sink.push(
            diagnostics::UnknownCommand {
                err_span: span,
                advice,
            }
            .into(),
        );
        return;
    };
    if command.has_arg() && command_token.argument == None {
        sink.push(diagnostics::MissingCommandArgument { err_span: span }.into());
    }
}

pub fn lint_string<'bump>(
    bump: &'bump Bump,
    decoded: &'bump str,
    style: MessageCommandStyle,
    source: AnyStringSource,
    index: u32,
) -> Result<(), LineReport> {
    if !source.contains_commands() {
        return Ok(());
    }

    let mut tokens = Vec::new_in(bump);
    super::parse(style, decoded, &mut tokens);

    // first, run diagnostics with fake spans to check if there are any errors
    let mut diagnostics = std::vec::Vec::new();
    for token in tokens {
        diagnose(
            &mut diagnostics,
            token,
            SourceSpan::new(SourceOffset::from(0), 0),
        );
        if !diagnostics.is_empty() {
            break;
        }
    }

    if diagnostics.is_empty() {
        return Ok(());
    }

    // now collect the real spans

    let mut spanned_tokens = Vec::new_in(bump);
    super::parse(style, decoded, &mut spanned_tokens);

    let mut diagnostics = std::vec::Vec::new();
    for super::SpannedMessageToken { token, start, end } in spanned_tokens {
        diagnose(
            &mut diagnostics,
            token,
            SourceSpan::new(SourceOffset::from(start), end - start),
        );
    }

    let report = LineReport {
        src: decoded.to_string(),
        index,
        diagnostics,
    };

    Err(report)
}
