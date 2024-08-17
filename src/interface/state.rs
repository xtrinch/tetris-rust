#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Paused,
    SoftDropping,
    LockingDown,
    LockedDown,
    TickingDown,
}
