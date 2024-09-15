#![no_std]
pub use fsm_macro::fsm;

// A marker trait for an event
pub trait Event { }

// A marker trait for a state
pub trait State { }

// A marker pub trait for a state machine
pub trait SM {
    type State;
}

// A trait for tranisioning between state in the state machine
pub trait Transition<E: Event> {
    type SM: SM;

    fn t(self, _e: E) -> Self::SM;
}

// A trait to convert from a state machine to an enum
pub trait ToEnum {
    type Repr;

    fn to_enum(self) -> Self::Repr;
}

// A trait to convert from a state machine to an enum
pub trait ToMemEnum {
    type Repr;
    type Mem;

    fn to_enum(self, mem: Self::Mem) -> Self::Repr;
}

// A marker trait for an entry point state
pub trait EntryPoint { }

// A trait defining an entrypoint event for state S
pub trait Init<S: State + EntryPoint> {
    type SM: SM<State = S>;
    
    fn init() -> Self::SM;
}

