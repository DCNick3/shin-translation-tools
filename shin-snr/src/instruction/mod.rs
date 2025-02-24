mod dispatch;
mod react;

pub use dispatch::decode_instr;
pub use react::react_instr;

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
    /// Inverted get table (index of matching entry in table)
    igt,

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

    SAVEINFO,
    // only on old
    MOVIE,
    AUTOSAVE,
    EVBEGIN,
    EVEND,
    RESUMESET,
    RESUME,

    TROPHY,
    UNLOCK,

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
    TIPSGET,
    CHARSEL,
    OTSUGET,
    CHART,
    SNRSEL,
    KAKERA,
    KAKERAGET,
    QUIZ,
    FAKESELECT,

    // this is the last thing in the opcode space
    DEBUGOUT,
}
