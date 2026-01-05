use std::{cell::Cell, fmt};

use hex_slice::AsHex;
use shin_versions::LengthKind;

use crate::operation::arena::{Number, Offset, Register};

#[derive(Debug, Copy, Clone)]
pub enum NumberArrayKind {
    Dense,
    Padded,
}

pub enum OperationElementRepr<'a, 'snr> {
    U8(u8),
    U16(u16),
    U32(u32),

    Operation(u8),
    Condition(u8),
    Expression(&'a [u8], &'a [Number]),

    Register(Register),
    RegisterArray(LengthKind, &'a [Register]),

    Offset(Offset),
    OffsetArray(LengthKind, &'a [Offset]),

    Number(Number),
    OptionalNumber(Option<Number>),
    NumberArray(LengthKind, NumberArrayKind, &'a [Number]),
    BitmaskNumberArray(u8, &'a [Number]),

    String(LengthKind, &'snr [u8]),
    StringArray(LengthKind, &'snr [u8]),

    // special kids used only for single commands
    HiguSuiWipeArg(u8, u8, &'a [Number]),
}

struct NumberFmt(u32);

impl fmt::Debug for NumberFmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#010X}", self.0)
    }
}
struct RegFmt(u16);

impl fmt::Debug for RegFmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#06X}", self.0)
    }
}

struct OffsetFmt(u32);
impl fmt::Debug for OffsetFmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#08X}", self.0)
    }
}

struct SeqFmt<I>(Cell<Option<I>>);

impl<I> SeqFmt<I> {
    pub fn new(iter: impl IntoIterator<IntoIter = I>) -> Self {
        SeqFmt(Cell::new(Some(iter.into_iter())))
    }
}

impl<I, T> fmt::Debug for SeqFmt<I>
where
    I: Iterator<Item = T>,
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, item) in (0..).zip(self.0.replace(None).unwrap()) {
            if i != 0 {
                write!(f, ", ")?;
            }
            fmt::Debug::fmt(&item, f)?
        }

        Ok(())
    }
}

impl fmt::Debug for OperationElementRepr<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use OperationElementRepr::*;

        match *self {
            U8(v) => {
                write!(f, "{v:#04X}u8")
            }
            U16(v) => {
                write!(f, "{v:#06X}u16")
            }
            U32(v) => {
                write!(f, "{v:#010X}u32")
            }
            Operation(op) => {
                write!(f, "{op}op")
            }
            Condition(cond) => {
                write!(f, "{cond}cd")
            }
            Expression(tokens, numbers) => {
                write!(
                    f,
                    "exp{{[{:#02X?}]/[{:?}]}}",
                    SeqFmt::new(tokens),
                    SeqFmt::new(numbers.iter().copied().map(NumberFmt))
                )
            }
            Register(reg) => {
                write!(f, "{:?}r", RegFmt(reg))
            }
            RegisterArray(_, regs) => {
                write!(f, "r[{:?}]", SeqFmt::new(regs.iter().copied().map(RegFmt)))
            }
            Offset(offset) => {
                write!(f, "{:?}j", OffsetFmt(offset))
            }
            OffsetArray(_, offsets) => {
                write!(
                    f,
                    "j[{:?}]",
                    SeqFmt::new(offsets.iter().copied().map(OffsetFmt))
                )
            }
            Number(number) => {
                write!(f, "{:?}rn", NumberFmt(number))
            }
            OptionalNumber(opt_number) => match opt_number {
                Some(number) => write!(f, "{:?}ron", NumberFmt(number)),
                None => write!(f, "<>ron"),
            },
            NumberArray(_, _, numbers) => {
                write!(
                    f,
                    "rn[{:?}]",
                    SeqFmt::new(numbers.iter().copied().map(NumberFmt))
                )
            }
            BitmaskNumberArray(mask, numbers) => {
                write!(
                    f,
                    "bm{{{:#01X}/[{:?}]}}",
                    mask,
                    SeqFmt::new(numbers.iter().copied().map(NumberFmt))
                )
            }
            String(_, s) => {
                write!(f, "s{:x}", s.as_hex())
            }
            StringArray(_, sa) => {
                write!(f, "sa{:x}", sa.as_hex())
            }
            HiguSuiWipeArg(b1, b2, numbers) => {
                write!(
                    f,
                    "higu_sui_wipe{{b1: {:#04X}, b2: {:#04X}, numbers: [{:?}]}}",
                    b1,
                    b2,
                    SeqFmt::new(numbers.iter().copied().map(NumberFmt))
                )
            }
        }
    }
}
