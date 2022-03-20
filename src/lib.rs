use serde::{Serialize, Deserialize};
use enumflags2::bitflags;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[repr(u64)]
pub enum FromClientMessage {
    W = 1 << 0,
    A = 1 << 1,
    D = 1 << 2,
    S = 1 << 3,
    Up = 1 << 4,
    Left = 1 << 5,
    Right = 1 << 6,
    Down = 1 << 7,
    Q = 1 << 8,
    E = 1 << 9,
    P = 1 << 10,
    O = 1 << 11,
    U = 1 << 12,
    Y = 1 << 13,
    I = 1 << 14,
    K = 1 << 15,
    L = 1 << 16,
    J = 1 << 17,
    Connect = 1 << 18
}


#[macro_export]
macro_rules! set_flag {
    ($n:expr, $f:expr) => {
        $n |= $f
    };
}
#[macro_export]
macro_rules! clr_flag {
    ($n:expr, $f:expr) => {
        n &= f
    }
}
#[macro_export]
macro_rules! tgl_flag {
    ($n:expr, $f:expr) => {
        n ^= f
    }
}
#[macro_export]
macro_rules! chk_flag {
    (&n:expr, &f:expr) => {
        n & f
    }
}