
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    Ident,
    Error
};

use std::collections::HashSet;

use crate::fsm::{
    event::{
        EDefinition,
        Events,
        Event
    },
    state::State,
    init::Inits,
    memory::{
        MemDefBlk,
        MemDef,
        MemDefs,
        StateMems,
        StateMem
    }
};

pub(crate) struct Machine {
    name: Ident,
    states: StateMems,
    inits: Inits,
    events: Events,
    variants: MemDefs
}

impl Parse for Machine {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut states: HashSet<State>;
        let mut events: HashSet<Event> = HashSet::new();

        let name: Ident = input.parse()?;

        let machine_blk;
        braced!(machine_blk in input);

        if !input.is_empty() {
            return Err(input.error(format!("Unexpected tokens after parsing.")));
        } 

        let inits: Inits = machine_blk.parse()?;
        states = inits.iter().cloned().map(|init| init.into()).collect();

        let MemDefBlk {
            states: mem_states,
            mem_defs
        } = machine_blk.parse()?;

        let mut mem_defs: Vec<MemDef> = mem_defs.into();

        while !machine_blk.is_empty() {
            let EDefinition {states: event_states, event} = machine_blk.parse()?;

            if let Some(first) = events.get(&event) {
                let mut err = Error::new_spanned(&event.name, format!{"Duplicate event: {}", event.name});
                err.combine(Error::new_spanned(&first.name, format!{"First declared here"}));
                return Err(err);
            }

            events.insert(event);
            states.extend(event_states);
        }

        mem_defs.extend(states.difference(&mem_states.into()).cloned().map(|s| s.into()));

        let state_mems: Vec<StateMem> = mem_defs.iter().cloned().map(|md| md.into()).collect();

        Ok (
            Machine {
                name,
                inits,
                states: state_mems.into(), 
                events: events.into(),
                variants: mem_defs.into()
            }
        )
    }
}

impl ToTokens for Machine {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let inits = &self.inits;
        let states = &self.states;
        let events = &self.events;
        let variants = &self.variants;

        tokens.extend(quote! {
            use fsm::{Transition, Init, ToEnum, ToMemEnum};
            mod #name {
                use core::marker::PhantomData;
                use fsm::{Event, State, SM, Transition, EntryPoint, Init, ToEnum, ToMemEnum};

                #[derive(Clone)]
                pub struct FSM<S: State> {
                    _s: PhantomData<S>,
                }

                impl<S: State> SM for FSM<S> {
                    type State = S;
                }

                impl<S: State + EntryPoint> FSM<S> {
                    fn init() -> FSM<S> {
                        FSM {
                            _s: PhantomData
                        }
                    }
                }

                impl<S: State + EntryPoint> Init<S> for FSM<S> {
                    type SM = FSM<S>;

                    fn init() -> Self::SM {
                        FSM {
                            _s: PhantomData
                        }
                    }
                }

                #states
                #inits
                #events
                pub enum Variants {
                    #variants
                }
            }
        });
    }
}
