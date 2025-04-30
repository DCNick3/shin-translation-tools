use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, minicbor::Encode, minicbor::Decode)]
pub struct FullLayoutParamsVita {
    #[n(0)]
    pub width: u32,
    #[n(1)]
    pub messagebox_type: u32,
    #[n(2)]
    pub font_scale: f32,
    #[n(3)]
    pub default_scale: i32,
    #[n(4)]
    pub default_color: i32,
    #[n(5)]
    pub default_draw_speed: i32,
    #[n(6)]
    pub default_fade: i32,
}

#[derive(Copy, Clone, Debug, minicbor::Encode, minicbor::Decode)]
pub enum FullLayoutParams {
    #[n(1)]
    Vita(#[n(0)] FullLayoutParamsVita),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, minicbor::Encode, minicbor::Decode)]
pub enum CharKind {
    #[n(0)]
    Regular = 0,
    #[n(1)]
    Furigana = 1,
    #[n(2)]
    BoldDot = 2,
}

#[derive(Copy, Clone, Debug, minicbor::Encode, minicbor::Decode)]
pub struct CharSummary {
    #[n(0)]
    pub index: u32,
    #[n(1)]
    pub codepoint: u16,
    #[n(2)]
    pub kind: CharKind,
    #[n(3)]
    pub has_furigana: bool,
    #[n(4)]
    pub has_bold_dot: bool,
    // #[n(5)] pub time2: f32,
    // #[n(6)] pub time1: f32,
    #[n(7)]
    pub pos_x: f32,
    #[n(8)]
    pub pos_y: f32,
    #[n(9)]
    pub scale: f32,
    #[n(10)]
    pub width: f32,
    #[n(11)]
    pub height: f32,
    #[n(12)]
    pub cant_be_at_line_start: bool,
    #[n(13)]
    pub cant_be_at_line_end: bool,
}

impl Display for CharSummary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let kind = match self.kind {
            CharKind::Regular => "r",
            CharKind::Furigana => "f",
            CharKind::BoldDot => "b",
        };

        let cp_unicode = shin_text::decode_sjis_codepoint(self.codepoint, false);

        fn b(bool: bool) -> &'static str {
            if bool {
                "X"
            } else {
                "_"
            }
        }

        write!(
            f,
            concat!(
                "{:>3} {:04x} `{}` k={} f={} b={} ",
                // "t2={:>6.1} t1={:>6.1} ",
                "x={:>6.1} y={:>6.1} s={:>4.2} w={:>4.1} h={:>4.1} ",
                "cbls={} cble={}"
            ),
            self.index,
            self.codepoint,
            cp_unicode,
            kind,
            b(self.has_furigana),
            b(self.has_bold_dot),
            // self.time2,
            // self.time1,
            self.pos_x,
            self.pos_y,
            self.scale,
            self.width,
            self.height,
            b(self.cant_be_at_line_start),
            b(self.cant_be_at_line_end),
        )
    }
}

#[derive(Debug, minicbor::Encode, minicbor::Decode)]
pub enum Event {
    #[n(0)]
    SetupParams(#[n(0)] FullLayoutParams),
    #[n(1)]
    NewMessage(#[n(0)] Vec<u8>),
    #[n(2)]
    PreFinalizeParagraph(#[n(0)] Vec<CharSummary>),
    #[n(3)]
    FinalizeUpTo {
        #[n(0)]
        index: u32,
        #[n(1)]
        hard: bool,
    },
    #[n(4)]
    Finish(#[n(0)] Vec<CharSummary>),
}

pub fn parse_dump(dump: &[u8]) -> Vec<Event> {
    minicbor::decode(dump).expect("Failed to decode dump")
}
