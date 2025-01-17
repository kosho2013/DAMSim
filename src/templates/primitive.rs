use core::fmt;

use dam::context_tools::*;
use dam::templates::ops::*;
use dam::types::StaticallySized;
use dam::RegisterALUOp;

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub enum Token<ValType, StopType> {
    Val(ValType),
    Stop(StopType),
    Empty,
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Repsiggen {
    #[default]
    Repeat,
    Stop,
    Done,
}

pub trait Exp {
    fn exp(self) -> Self;
}

RegisterALUOp!(ALUIdentityOp_1input, |(i0), ()| [i0], T: DAMType);
RegisterALUOp!(ALUIdentityOp_2input, |(i0, i1), ()| [i0, i1], T: DAMType);
RegisterALUOp!(ALUIdentityOp_3input, |(i0, i1, i2), ()| [i0, i1, i2], T: DAMType);
RegisterALUOp!(ALUIdentityOp_4input, |(i0, i1, i2, i3), ()| [i0, i1, i2, i3], T: DAMType);
RegisterALUOp!(ALUIdentityOp_5input, |(i0, i1, i2, i3, i4), ()| [i0, i1, i2, i3, i4], T: DAMType);
RegisterALUOp!(ALUIdentityOp_6input, |(i0, i1, i2, i3, i4, i5), ()| [i0, i1, i2, i3, i4, i5], T: DAMType);
RegisterALUOp!(ALUIdentityOp_32input, |(i0, i1, i2, i3, i4, i5, i6, i7, i8, i9, i10, i11, i12, i13, i14, i15, i16, i17, i18, i19, i20, i21, i22, i23, i24, i25, i26, i27, i28, i29, i30, i31), ()| [i0, i1, i2, i3, i4, i5, i6, i7, i8, i9, i10, i11, i12, i13, i14, i15, i16, i17, i18, i19, i20, i21, i22, i23, i24, i25, i26, i27, i28, i29, i30, i31], T: DAMType);
RegisterALUOp!(ALUExpOp, |(i0), ()| [i0.exp()], T: DAMType + Exp);


impl<ValType: DAMType, StopType: DAMType> Exp for Token<ValType, StopType>
where
    ValType: Exp,
{
    fn exp(self) -> Self {
        match self {
            Token::Val(val) => Token::Val(val.exp()),
            _ => self,
        }
    }
}

impl<T: num::Float> Exp for T {
    fn exp(self) -> Self {
        num::Float::exp(self)
    }
}

impl<ValType: DAMType, StopType: DAMType> fmt::Debug for Token<ValType, StopType> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Val(val) => {
                write!(f, "{:#?}", val)
            }
            Token::Stop(tkn) => {
                write!(f, "S{:#?}", tkn)
            }
            Token::Empty => {
                write!(f, "N")
            }
            Token::Done => {
                write!(f, "D")
            }
        }
    }
}

impl<ValType, StopType: core::str::FromStr> TryFrom<&str> for Token<ValType, StopType> {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.starts_with('D') {
            Ok(Self::Done)
        } else if value.starts_with('N') {
            Ok(Self::Empty)
        } else if let Some(stripped) = value.strip_prefix('S') {
            stripped.parse().map(Self::Stop).map_err(|_| ())
        } else {
            Err(())
        }
    }
}

impl TryFrom<&str> for Repsiggen {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.starts_with('R') {
            Ok(Self::Repeat)
        } else if value.starts_with('S') {
            Ok(Self::Stop)
        } else if value.starts_with('D') {
            Ok(Self::Done)
        } else {
            Err(())
        }
    }
}

#[macro_export]
macro_rules! token_vec {
    [$toktype: tt; $stoptype: tt; $($val:expr),*] => {
        ({
            let hl = frunk::hlist![$($val),*];
            let mapped = hl.map(
                frunk::poly_fn![
                    |f: &'static str| -> Token<$toktype, $stoptype> {Token::<$toktype, $stoptype>::try_from(f).unwrap()},
                    |v: $toktype| -> Token<$toktype, $stoptype> {Token::<$toktype, $stoptype>::Val(v)},
                    ]
            );
            let result = vec![];
            mapped.foldl(|mut acc: Vec<_>, x| {acc.push(x); acc}, result)
        })
    };
}

#[macro_export]
macro_rules! repsig_vec {
    [$($val:expr),*] => {
        ({
            let mut res = Vec::new();
            $(
                {
                    res.push(Repsiggen::try_from($val).unwrap());
                }
            )*
            res
        })
    };
}

impl<ValType: DAMType, StopType: DAMType> std::ops::Neg for Token<ValType, StopType>
where
    ValType: std::ops::Neg<Output = ValType>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Token::Val(val) => Token::Val(val.neg()),
            _ => self,
        }
    }
}

fn tmp() {
    let _ = token_vec![u16; u16; 1, 2, 3, "S0", 4, 5, 6, "S1", "D"];
    let _ = repsig_vec!("R", "R", "S", "D");
}

impl<ValType: Default, StopType: Default> Default for Token<ValType, StopType> {
    fn default() -> Self {
        Token::Val(ValType::default())
    }
}

impl<ValType: DAMType, StopType: DAMType> DAMType for Token<ValType, StopType> {
    fn dam_size(&self) -> usize {
        2 + match self {
            Token::Val(val) => val.dam_size(),
            Token::Stop(stkn) => stkn.dam_size(),
            Token::Empty => 0,
            Token::Done => 0,
        }
    }
}

impl StaticallySized for Repsiggen {
    const SIZE: usize = 2;
}