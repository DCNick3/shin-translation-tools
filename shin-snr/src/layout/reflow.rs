use bumpalo::{collections::Vec, Bump};
use shin_font::FontMetrics;

use crate::layout::{
    layouter::{self, GameLayoutInfo, LightLayouter, PushResult},
    message_parser::{CommandToken, MessageToken},
};

pub fn reflow_paragraph(
    layout_info: GameLayoutInfo,
    base_index: usize,
    break_points: &mut Vec<usize>,
    commands: &[layouter::Command],
) {
    let mut break_iter = super::linebreak::LineBreakIterator::new(commands).peekable();
    let mut base_x = 0.0;
    let mut last_seen_candidate_breakpoint = None;

    for (index, command) in (0..).zip(commands) {
        if let Some((break_index, _is_hard)) =
            break_iter.next_if(|&(break_index, _)| index >= break_index)
        {
            last_seen_candidate_breakpoint = Some(break_index);
        }

        if let layouter::Command::Char(c) = command {
            if c.pos_x + c.width - base_x > layout_info.width {
                // need a line break
                if let Some(breakpoint) = last_seen_candidate_breakpoint {
                    last_seen_candidate_breakpoint = None; // do not re-use the same breakpoint!
                    break_points.push(base_index + breakpoint);
                    let layouter::Command::Char(c) = &commands[breakpoint] else {
                        unreachable!("break points should only happen at chars")
                    };
                    base_x = c.pos_x + c.width;
                } else {
                    // no valid breakpoint, force a break right here
                    break_points.push(base_index + index);
                    base_x = c.pos_x + c.width;
                }
            }
        }
    }
}

pub fn reflow_message<'s>(
    bump: &Bump,
    font_metrics: &FontMetrics,
    layout_info: GameLayoutInfo,
    tokens_in: &[MessageToken<'s>],
    tokens_out: &mut Vec<MessageToken<'s>>,
) {
    tokens_out.reserve(tokens_in.len() + 8);
    let mut layouter = LightLayouter::new(bump, font_metrics, layout_info);

    let mut break_points = Vec::with_capacity_in(16, bump);

    fn flush_paragraph<'s>(
        break_points: &mut Vec<usize>,
        layout_info: GameLayoutInfo,
        para_commands: &[layouter::Command<'s>],
        commands_out: &mut Vec<MessageToken<'s>>,
    ) {
        break_points.clear();
        reflow_paragraph(layout_info, 0, break_points, para_commands);

        let mut break_iter = break_points.iter().copied().peekable();

        for (index, command) in (0..).zip(para_commands) {
            if break_iter
                .next_if(|&break_index| index >= break_index)
                .is_some()
            {
                commands_out.push(MessageToken::Command(CommandToken {
                    command: 'r',
                    argument: None,
                }));
            }

            commands_out.push(command.lower())
        }
    }

    for &command in tokens_in {
        if let PushResult::ParagraphComplete = layouter.push(command) {
            flush_paragraph(
                &mut break_points,
                layout_info,
                layouter.peek_buffer(),
                tokens_out,
            );
            layouter.reset_buffer();
        }
    }
    flush_paragraph(
        &mut break_points,
        layout_info,
        layouter.peek_buffer(),
        tokens_out,
    );
}

#[cfg(test)]
mod test {
    use bumpalo::{collections::Vec, Bump};
    use shin_versions::{MessageCommandStyle, ShinVersion, SjisMessageFixupPolicy};

    use crate::layout::{
        layouter, layouter::GameLayoutInfo, message_parser, reflow::reflow_paragraph,
    };

    #[test]
    fn smoke() {
        let params = GameLayoutInfo::for_version(ShinVersion::HigurashiSui).unwrap();

        let message = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

        let commands =
            layouter::test::layout_message(MessageCommandStyle::Escaped, params.into(), message);

        let bump = Bump::new();
        let mut break_points = Vec::new_in(&bump);
        let mut base_index = 0;
        for paragraph in &commands {
            reflow_paragraph(params, base_index, &mut break_points, paragraph);
            base_index += paragraph.len();
        }

        assert_eq!(break_points, [40, 90, 141, 191, 242, 295, 345, 397,])
    }

    fn reflow_message(message: &str) -> String {
        let params = GameLayoutInfo::for_version(ShinVersion::HigurashiSui).unwrap();

        let bump = Bump::new();

        let mut tokens = Vec::new_in(&bump);
        message_parser::parse(MessageCommandStyle::Escaped, message, &mut tokens);

        let mut tokens_out = Vec::new_in(&bump);
        super::reflow_message(
            &bump,
            &layouter::test::RESOURCES.metrics,
            params,
            &tokens,
            &mut tokens_out,
        );

        let mut s = String::new();
        message_parser::serialize(
            MessageCommandStyle::Escaped,
            SjisMessageFixupPolicy {
                fixup_command_arguments: false,
                fixup_character_names: false,
            },
            true,
            &tokens_out,
            &mut s,
        );

        s
    }

    #[test]
    fn smoke_message() {
        assert_eq!(reflow_message(
            "Chara Name@r@vaboba.Lorem ipsum dolor sit amet, @bsome latin bs.@<consectetur adipiscing@> elit."),
                     "Chara Name@r@vaboba.Lorem ipsum dolor sit @ramet, @bsome latin bs.@<consectetur adipiscing@> elit."
        );
        assert_eq!(reflow_message(
            "Chara Name@r@vaboba.Lorem ipsum dolor sit @bsome latin bs.@<amet, consectetur@> adipiscing elit."),
                     "Chara Name@r@vaboba.Lorem ipsum dolor sit @bsome latin bs.@<amet, consectetur@> @radipiscing elit."
        );
    }
}
