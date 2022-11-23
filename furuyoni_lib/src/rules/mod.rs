mod states;
pub use {
    states::ViewableOpponentState, states::ViewablePlayerState, states::ViewablePlayerStates,
    states::ViewableSelfState, states::ViewableState,
};

#[derive(Debug, Clone, Copy)]
pub enum Phase {
    Beginning,
    Main,
    End,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum PlayerPos {
    P1,
    P2,
}
