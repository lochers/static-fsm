#![no_std]
pub use fsm_macro::fsm;

// A marker trait for an event
pub trait Event { }

// A marker trait for a state
pub trait State { }

// A marker pub trait for a state machine
pub trait SM {
    type Event;
    type State;
}

// A trait for tranisioning between state in the state machine
pub trait Transition<E: Event> {

    type SM: SM;

    fn t(self, event:E) -> Self::SM;
}

// A trait to convert from a state machine to an enum
pub trait ToEnum {
    type Repr;

    fn to_enum(self) -> Self::Repr;
}

// A trait for all the entry points of state S
pub trait EntryPoints<S, E>
where
    S: State,
    E: EntryPoint<S, SM: SM<State = S, Event = E>> + Event
{
    type SM: SM<State = S, Event = E>;

    fn new() -> Self::SM;
}

// A trait defining an entrypoint event for state S
pub trait EntryPoint<S:State> {
    type Event: Event;
    type SM: SM<State = S, Event = Self::Event>;
    
    fn fsm() -> Self::SM;
}

