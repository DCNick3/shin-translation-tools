pub mod arena;
mod dispatch;
pub mod parse;
mod repr;
pub mod schema;
pub mod serialize;

pub use dispatch::decode_instr;
pub use repr::OperationElementRepr;

// we break the rust naming rules in order to
// 1. match game's COMMAND names
// 2. distinguish instructions (lowercase) from commands (UPPERCASE)
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    // Instructions
    // they do not affect the game state and are internal to the VM
    // these do not seem to change between versions
    // NOTE: not all implemented opcodes are implemented here, because I am lazy
    /// Unary operation
    uo,
    /// Binary operation
    bo,
    /// Expression (encoded in RPN)
    exp,
    /// Move many
    mm,
    /// Get table (table is array of numbers)
    gt,
    /// Set table (table is array of registers)
    st,
    /// Conditional jump
    jc,
    /// Unconditional jump
    j,
    /// Call subroutine
    gosub,
    /// Return from subroutine
    retsub,
    /// Jump table
    jt,
    pop,
    /// Call subroutine table
    gosubt,
    rnd,
    push,
    /// Call function
    call,
    /// Return from function
    r#return,
    /// Find Match in Table (index of first matching entry in table)
    fmt,
    /// Find Non-match in Table (index of first non-matching entry in table)
    ///
    /// The only difference from [Instruction::fmt] is the flipped equality check
    fnmt,
    /// Get the character ID corresponding to the passed Bustup info
    getbupid,

    // Commands
    // they yield to the game loop and are what affects the game state
    // they can be interpreted differently in different contexts (e.g. running the ADV vs building the log)
    // they do tend to change between versions
    // NOTE: I believe currently all Astral air opcodes are listed here
    EXIT,
    SGET,
    SSET,
    WAIT,
    // only on old
    KEYWAIT,
    MSGINIT,
    MSGSET,
    MSGWAIT,
    MSGSIGNAL,
    MSGSYNC,
    MSGCLOSE,
    MSGFACE,
    MSGCHECK,
    // seen only in higurashi-sui
    MSGQUAKE,
    // seen only in higurashi-hou-v2
    MSGHIDE,
    LOGSET,
    SELECT,
    WIPE,
    WIPEWAIT,

    BGMPLAY,
    BGMSTOP,
    BGMVOL,
    BGMWAIT,
    BGMSYNC,
    SEPLAY,
    SESTOP,
    SESTOPALL,
    SEVOL,
    SEPAN,
    SEWAIT,
    SEONCE,
    // Unknown instuction with no semantics (like, really, the handlers are empty)
    // but it has some arguments. Probably a leftover from even older version...
    UNK9B,
    VOICEPLAY,
    VOICESTOP,
    VOICEWAIT,

    SYSSE,

    SAVEINFO,
    // only on old
    MOVIE,
    AUTOSAVE,
    EVBEGIN,
    EVEND,
    RESUMESET,
    RESUME,
    SYSCALL,

    TROPHY,
    UNLOCK, // also in the game-specific section in Higurashi Hou

    // only on old
    LAYERCLEAR,
    LAYERINIT,
    LAYERLOAD,
    LAYERUNLOAD,
    LAYERCTRL,
    LAYERWAIT,
    LAYERSWAP,
    LAYERSELECT,
    MOVIEWAIT,
    // mid-version? Before we had planes, but no LAYERSWAP
    // I think konosuba and higurashi have this
    LAYERBACK,
    // "new" plane-related commands
    TRANSSET,
    TRANSWAIT,
    PAGEBACK,
    PLANESELECT,
    PLANECLEAR,
    MASKLOAD,
    MASKUNLOAD,

    // Game-specific commands
    // Alias Carnival
    ICOGET,
    STAGEINFO,
    ICOCHK,
    EMOTEWAIT,
    NAMED,
    BACKINIT,

    // White Eternity
    CHARCLEAR,
    CHARLOAD,
    CHARUNLOAD,
    CHARDISP,
    CHARCTRL,
    CHARWAIT,
    CHARMARK,
    CHARSYNC,

    // DC4
    CHATSET,

    // Konosuba
    SLEEP,
    VSET,

    // Higurashi Sui
    TIPSGET, // also shared by Higurashi Hou & Umineko
    CHARSEL,
    OTSUGET,    // also shared by Higurashi Hou
    CHART,      // also shared by Higurashi Hou
    SNRSEL,     // also shared by Higurashi Hou
    KAKERA,     // also shared by Higurashi Hou
    KAKERAGET,  // also shared by Higurashi Hou
    QUIZ,       // also shared by Higurashi Hou, Umineko and Gerokasu2
    FAKESELECT, // also shared by Higurashi Hou

    // Umineko
    CHARS,
    SHOWCHARS,
    NOTIFYSET,

    // Higurashi Hou
    FEELICON,
    CHARSELECT,

    // Higurashi Hou V2
    KGET,
    KSET,

    // this is the last thing in the opcode space
    DEBUGOUT,
}
