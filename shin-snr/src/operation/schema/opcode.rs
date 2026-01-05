use std::fmt;

use enum_map::Enum;
use shin_versions::{StringArrayKind, StringKind};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum)]
#[allow(non_camel_case_types)]
pub enum Instruction {
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
    /// Call subroutine table
    gosubt,
    rnd,
    push,
    pop,
    /// Call function
    call,
    /// Return from function
    r#return,
    /// Inverted get table (index of matching entry in table)
    fmt,
    /// Name is a placeholder, never encountered yet
    fnmt,
    /// Get the character ID corresponding to the passed Bustup info
    getbupid,
}
#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum)]
#[allow(clippy::upper_case_acronyms)]
pub enum Command {
    // Commands
    // they yield to the game loop and are what affects the game state
    // they can be interpreted differently in different contexts (e.g. running the ADV vs building the log)
    // they do tend to change between versions
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

impl TryFrom<Command> for StringKind {
    type Error = ();

    fn try_from(value: Command) -> Result<Self, Self::Error> {
        use Command::*;

        Ok(match value {
            SAVEINFO => StringKind::Saveinfo,
            SELECT => StringKind::Select,
            MSGSET => StringKind::Msgset,
            DEBUGOUT => StringKind::Dbgout,
            LOGSET => StringKind::Logset,
            VOICEPLAY => StringKind::Voiceplay,
            // Game-specific string kinds
            CHATSET => StringKind::Chatset,
            // Alias Carnival
            NAMED => StringKind::Named,
            STAGEINFO => StringKind::Stageinfo,
            _ => return Err(()),
        })
    }
}

impl TryFrom<Command> for StringArrayKind {
    type Error = ();

    fn try_from(value: Command) -> Result<Self, Self::Error> {
        use Command::*;

        Ok(match value {
            SELECT => StringArrayKind::SelectChoice,
            _ => return Err(()),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Enum)]
pub enum Opcode {
    Instruction(Instruction),
    Command(Command),
}

impl fmt::Debug for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::Instruction(instr) => fmt::Debug::fmt(instr, f),
            Opcode::Command(cmd) => fmt::Debug::fmt(cmd, f),
        }
    }
}
