use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum FromClientMessage {
    Connect,
    W,
    A,
    D,
    S,
    Up,
    Left,
    Right,
    Down,
    Q,
    E,
    P,
    O,
    U,
    Y,
    I,
    K,
    L,
    J,
    None,
}