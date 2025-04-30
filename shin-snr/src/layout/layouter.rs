use bumpalo::{collections::Vec, Bump};
use shin_font::FontMetrics;
use shin_versions::{MessageCommandStyle, ShinVersion};

use crate::layout::{
    layout_dump, message_parser,
    message_parser::{CommandToken, MessageCommand, MessageToken},
};

#[derive(Debug, Copy, Clone)]
pub struct Char {
    // earlier game versions perform layouting in Shift-JIS directly, while newer ones convert the message to UTF-8 first
    // I will opt to do the latter, since I already have a robust parser that only works with unicode
    pub codepoint: char,
    pub has_furigana: bool,
    pub pos_x: f32,
    pub scale: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Copy, Clone)]
pub enum Command<'bump> {
    Unparsed(CommandToken<'bump>),
    Char(Char),
}

impl<'bump> Command<'bump> {
    pub fn lower(self) -> MessageToken<'bump> {
        match self {
            Command::Unparsed(c) => MessageToken::Command(c),
            Command::Char(c) => MessageToken::Literal(c.codepoint),
        }
    }
}

pub enum PushResult {
    Nothing,
    ParagraphComplete,
}

pub struct LightLayouter<'bump, 'font, 's> {
    font_metrics: &'font FontMetrics,
    pos_x: f32,
    current_scale: f32,
    default_scale: f32,
    overall_scale_factor: f32,
    furigana_open: bool, // TODO: newer versions auto-close furigana
    furigana_start_x: f32,
    furigana_start_index: usize,
    furigana_content_size: f32,
    buffer: Vec<'bump, Command<'s>>,
}

pub fn parse_font_scale(int: u32) -> f32 {
    match int {
        ..10 => 0.1,
        10..200 => int as f32 * 0.01,
        200.. => 2.0,
    }
}

#[derive(Debug, Copy, Clone)]
pub struct GameLayoutInfo {
    pub default_scale: f32,
    pub overall_scale_factor: f32,
    pub width: f32,
}

impl GameLayoutInfo {
    pub fn for_version(version: ShinVersion) -> Option<Self> {
        match version {
            ShinVersion::HigurashiSui => Some(GameLayoutInfo {
                default_scale: 1.0,
                overall_scale_factor: 1.3333334,
                width: 1082.0,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LightLayouterParams {
    pub default_scale: f32,
    pub overall_scale_factor: f32,
}

impl LightLayouterParams {
    pub fn for_version(version: ShinVersion) -> Option<Self> {
        Some(GameLayoutInfo::for_version(version)?.into())
    }
}

impl From<GameLayoutInfo> for LightLayouterParams {
    fn from(value: GameLayoutInfo) -> Self {
        Self {
            default_scale: value.default_scale,
            overall_scale_factor: value.overall_scale_factor,
        }
    }
}

impl<'bump, 'font, 's> LightLayouter<'bump, 'font, 's> {
    pub fn new(
        bump: &'bump Bump,
        font_metrics: &'font FontMetrics,
        params: impl Into<LightLayouterParams>,
    ) -> Self {
        let LightLayouterParams {
            default_scale,
            overall_scale_factor,
        } = params.into();

        Self {
            font_metrics,
            pos_x: 0.0,
            current_scale: default_scale * overall_scale_factor,
            default_scale,
            overall_scale_factor,
            furigana_open: false,
            furigana_start_x: 0.0,
            furigana_start_index: 0,
            furigana_content_size: 0.0,
            buffer: Vec::with_capacity_in(72, bump),
        }
    }

    pub fn reset(&mut self) {
        self.pos_x = 0.0;
        self.current_scale = self.default_scale * self.overall_scale_factor;
        self.furigana_open = false;
    }

    fn push_char(&mut self, char: char) {
        let glyph_metrics = self
            .font_metrics
            .get_glyph_metrics(char)
            .expect("could not get glyph metrics");

        let font_info = self.font_metrics.get_info();

        let scale = self.current_scale;

        let width = glyph_metrics.advance_width as f32 * scale;
        let height = (font_info.ascent + font_info.descent) as f32 * scale;

        let char = Char {
            codepoint: char,
            has_furigana: self.furigana_open,
            pos_x: self.pos_x,
            scale,
            width,
            height,
        };
        self.pos_x += width;

        self.buffer.push(Command::Char(char));
    }

    fn measure_furigana(&self, content: &str) -> f32 {
        let mut width = 0.0;
        for c in content.chars() {
            let glyph_metrics = self
                .font_metrics
                .get_glyph_metrics(c)
                .expect("could not get glyph metrics");

            width += glyph_metrics.advance_width as f32 * 0.45;
        }
        width
    }

    pub fn push(&mut self, value: MessageToken<'s>) -> PushResult {
        match value {
            MessageToken::Command(c) => {
                let Some(command) = MessageCommand::parse(c.command) else {
                    // pass unsupported commands through
                    self.buffer.push(Command::Unparsed(c));
                    return PushResult::Nothing;
                };

                let result = match command {
                    MessageCommand::EnableLipsync
                    | MessageCommand::DisableLipsync
                    | MessageCommand::VoiceWait
                    | MessageCommand::SetFade
                    | MessageCommand::SetColor
                    | MessageCommand::NoFinalClickWait
                    | MessageCommand::ClickWait
                    | MessageCommand::VoiceVolume
                    | MessageCommand::TextSpeed
                    | MessageCommand::StartParallel
                    | MessageCommand::Voice
                    | MessageCommand::Wait
                    | MessageCommand::VoiceSync
                    | MessageCommand::Sync
                    | MessageCommand::CompleteSection
                    | MessageCommand::InstantTextStart
                    | MessageCommand::InstantTextEnd
                    | MessageCommand::BoldTextStart
                    | MessageCommand::BoldTextEnd => {
                        // we do not care about these
                        PushResult::Nothing
                    }

                    MessageCommand::RubiContent => {
                        let content = c
                            .argument
                            .expect("RubiContent command should have an argument");

                        self.furigana_content_size = self.measure_furigana(content);

                        PushResult::Nothing
                    }
                    MessageCommand::RubiBaseStart => {
                        self.furigana_start_x = self.pos_x;
                        self.furigana_start_index = self.buffer.len();
                        self.furigana_open = true;

                        PushResult::Nothing
                    }
                    MessageCommand::RubiBaseEnd => {
                        self.furigana_open = false;

                        let furigana_width = self.furigana_content_size;
                        let base_width = self.pos_x - self.furigana_start_x;
                        let base_range = self.furigana_start_index..self.buffer.len();
                        let base_char_count = self.buffer[base_range.clone()]
                            .iter()
                            .filter(|c| matches!(c, Command::Char(_)))
                            .count();

                        if base_char_count > 0 && furigana_width > base_width {
                            // base text needs reflowing to fit the furigana text
                            let reflow_space_around =
                                (furigana_width - base_width) / (base_char_count + 1) as f32;
                            let mut reflow_position = reflow_space_around;
                            for command in &mut self.buffer[base_range] {
                                let Command::Char(char) = command else {
                                    continue;
                                };
                                char.pos_x += reflow_position;
                                reflow_position += reflow_space_around;
                            }
                            self.pos_x = self.furigana_start_x + furigana_width;
                        }

                        PushResult::Nothing
                    }
                    MessageCommand::Unicode => {
                        // NOTE: older engine versions do not (and cannot) support this command
                        todo!()
                    }
                    MessageCommand::FontScale => {
                        // TODO: do we want to be more robust here?
                        // for example, negative values (which correspond to invalid values in this version) map to default font scale
                        let scale = c
                            .parse_int_arg()
                            .expect("could not parse FontScale argument");
                        self.current_scale = parse_font_scale(scale) * self.overall_scale_factor;

                        PushResult::Nothing
                    }
                    MessageCommand::Newline => PushResult::ParagraphComplete,
                };

                if !matches!(command, MessageCommand::Unicode) {
                    self.buffer.push(Command::Unparsed(c));
                }

                result
            }
            MessageToken::Literal(c) => {
                self.push_char(c);
                PushResult::Nothing
            }
        }
    }

    pub fn reset_buffer(&mut self) {
        self.pos_x = 0.0;
        self.buffer.clear();
    }

    pub fn peek_buffer(&self) -> &[Command<'s>] {
        &self.buffer
    }

    pub fn take_buffer(&mut self) -> Vec<'bump, Command<'s>> {
        self.pos_x = 0.0;
        let mut res = Vec::new_in(self.buffer.bump());
        std::mem::swap(&mut self.buffer, &mut res);
        res
    }
}

fn validate_paragraph(
    reference_paragraph: &[layout_dump::CharSummary],
    actual_paragraph: &[Command],
) {
    // only take chars, ignore the commands
    let actual_paragraph = actual_paragraph
        .iter()
        .filter_map(|c| match c {
            Command::Unparsed(_) => None,
            Command::Char(c) => Some(c),
        })
        .collect::<std::vec::Vec<_>>();

    // only take regular chars
    let reference_paragraph = reference_paragraph
        .iter()
        .filter(|&c| c.kind == layout_dump::CharKind::Regular)
        .collect::<std::vec::Vec<_>>();

    assert_eq!(reference_paragraph.len(), actual_paragraph.len());

    for (reference, actual) in reference_paragraph.into_iter().zip(actual_paragraph) {
        // reference index is not very interesting, because it doesn't include stuff like changing the font size or newlines
        // assert_eq!(reference.index, index_base + actual_char_index as u32);

        assert_eq!(
            shin_text::decode_sjis_codepoint(reference.codepoint, true),
            actual.codepoint
        );

        assert_eq!(reference.has_furigana, actual.has_furigana);
        assert_eq!(reference.pos_x, actual.pos_x);
        assert_eq!(reference.scale, actual.scale);
        assert_eq!(reference.width, actual.width);
        assert_eq!(reference.height, actual.height);
    }
}

#[expect(unused_variables, unused_assignments)] // current_message is useful for debugging
pub fn validate_light_layouter_against_dump(
    font_metrics: &FontMetrics,
    dump: &[layout_dump::Event],
) {
    let bump = Bump::new();

    let mut layouter = LightLayouter::new(
        &bump,
        font_metrics,
        LightLayouterParams {
            default_scale: 0.0,
            overall_scale_factor: 0.0,
        },
    );

    let mut paragraph_buffers_iter = std::vec::Vec::new().into_iter();
    let mut current_message = String::new();

    for event in dump {
        match event {
            layout_dump::Event::SetupParams(layout_dump::FullLayoutParams::Vita(params)) => {
                layouter = LightLayouter::new(
                    &bump,
                    font_metrics,
                    LightLayouterParams {
                        default_scale: parse_font_scale(params.default_scale.try_into().unwrap()),
                        overall_scale_factor: params.font_scale,
                    },
                );
            }
            layout_dump::Event::NewMessage(message) => {
                layouter.reset();

                let message = shin_text::decode_sjis_zstring(&bump, message, true).unwrap();

                current_message = message.to_string();

                let mut tokens = std::vec::Vec::new();
                message_parser::parse(MessageCommandStyle::Unescaped, message, &mut tokens);

                let mut line_buffers = std::vec::Vec::new();
                for token in tokens {
                    if let PushResult::ParagraphComplete = layouter.push(token) {
                        line_buffers.push(layouter.take_buffer());
                    }
                }
                // the last push won't complete a line, so we have to take the leftovers ourselves
                line_buffers.push(layouter.take_buffer());

                paragraph_buffers_iter = line_buffers.into_iter();
            }
            layout_dump::Event::PreFinalizeParagraph(reference_line) => {
                let actual_paragraph = paragraph_buffers_iter
                    .next()
                    .expect("Not enough actual lines to validate");
                validate_paragraph(reference_line, &actual_paragraph);
            }
            layout_dump::Event::FinalizeUpTo { .. } => {}
            layout_dump::Event::Finish(_) => {}
        }
    }
}

#[cfg(test)]
pub(super) mod test {
    use std::sync::LazyLock;

    use bumpalo::Bump;
    use shin_font::FontMetrics;
    use shin_versions::{MessageCommandStyle, ShinVersion};

    use crate::layout::{
        layout_dump,
        layouter::{Command, LightLayouter, LightLayouterParams, PushResult},
        message_parser,
    };

    pub struct Resources {
        pub metrics: FontMetrics,
        pub layout_dump: Vec<layout_dump::Event>,
    }

    pub static RESOURCES: LazyLock<Resources> = LazyLock::new(|| {
        let metrics = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../test-assets/higurashi-sui/font.fnt");
        let mut metrics = std::fs::File::open(&metrics).unwrap();
        let metrics = FontMetrics::from_font0(&mut metrics).unwrap();

        let layout_dump = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../test-assets/higurashi-sui/layout-mini.cbor");
        let layout_dump = std::fs::read(&layout_dump).unwrap();
        let layout_dump = layout_dump::parse_dump(&layout_dump);

        Resources {
            metrics,
            layout_dump,
        }
    });

    pub fn layout_message(
        style: MessageCommandStyle,
        params: LightLayouterParams,
        message: &str,
    ) -> Vec<Vec<Command>> {
        let mut tokens = Vec::new();
        message_parser::parse(style, message, &mut tokens);

        let bump = Bump::new();
        let mut layouter = LightLayouter::new(&bump, &RESOURCES.metrics, params);

        let mut line_buffers = Vec::new();

        for token in tokens {
            if let PushResult::ParagraphComplete = layouter.push(token) {
                line_buffers.push(layouter.take_buffer().into_iter().collect());
            }
        }

        line_buffers.push(layouter.take_buffer().into_iter().collect());

        line_buffers
    }

    #[test]
    fn smoke() {
        let message = "官僚rvS13/00/552100009.「ハリガネムシに寄生されたバッタのbじゅすい.<入水>自殺がいい例ですな。kvS13/00/552100010.ハリガネムシは充分に成長すると水棲となります。kvS13/00/552100011.その際、宿主であるバッタを水辺へ誘導して溺死させ、その体内から水中へ脱出する」";
        // let message = "!H!e!l!l!o! !w!o!r!l!d!!! !I! !a!m! !a! !s!t!r!i!n!g!,! !w!i!t!h! !s!o!m!e! !c!o!m!m!a!s!.";
        let paragraphs = layout_message(
            MessageCommandStyle::Unescaped,
            LightLayouterParams::for_version(ShinVersion::HigurashiSui).unwrap(),
            message,
        );

        println!("{:?}", paragraphs);
    }

    #[test]
    fn test_against_dump() {
        super::validate_light_layouter_against_dump(&RESOURCES.metrics, &RESOURCES.layout_dump);
    }
}
