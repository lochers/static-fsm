
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    Ident,
    Error
};

use std::collections::HashSet;

use crate::fsm::event::{
    EDefinition,
    Event
};
use crate::fsm::event::Events;
use crate::fsm::state::{
    States,
    State
};
use crate::fsm::variant::{
    Variants,
    Variant
};

pub(crate) struct Machine {
    name: Ident,
    states: States,
    events: Events,
    variants: Variants
}

impl Parse for Machine {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut states: HashSet<State> = HashSet::new();
        let mut events: HashSet<Event> = HashSet::new();
        let mut variants: Vec<Variant> = Vec::new();

        let name: Ident = input.parse()?;

        let machine_blk;
        braced!(machine_blk in input);

        if !input.is_empty() {
            return Err(input.error(format!("Unexpected tokens after parsing.")));
        } 

        while !machine_blk.is_empty() {
            let EDefinition {states: event_states, event} = machine_blk.parse()?;

            if let Some(first) = events.get(&event) {
                let mut err = Error::new_spanned(&event.name, format!{"Duplicate event: {}", event.name});
                err.combine(Error::new_spanned(&first.name, format!{"First declared here"}));
                return Err(err);
            }

            variants.extend(event.variants.iter().cloned().map(|ev| ev.into()));
            events.insert(event);
            states.extend(event_states);
        }

        Ok (
            Machine {
                name,
                states: states.into(), 
                events: events.into(),
                variants: variants.into()
            }
        )
    }
}

impl ToTokens for Machine {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let states = &self.states;
        let events = &self.events;
        let variants = &self.variants;

        tokens.extend(quote! {
            use fsm::{SM, Transition, EntryPoints, ToEnum};
            mod #name {
                use fsm::{Event, State, SM, Transition, EntryPoint, EntryPoints, ToEnum};

                #[derive(Clone)]
                pub struct FSM<S: State, E: Event> {
                    s: S,
                    e: E
                }

                impl<S: State, E: Event> SM for FSM<S, E> {
                    type State = S;
                    type Event = E;
                }

                impl<S: State, E> EntryPoints<S, E> for FSM<S, E>
                where
                    E: EntryPoint<S, SM = FSM<S,E>> + Event
                {
                    type SM = FSM<S,E>;

                    fn new() -> Self::SM {
                        E::fsm()
                    }
                }

                #states
                #events
                #variants
            }
        });
    }
}
