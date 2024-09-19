
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    Ident,
    Error
};

use std::collections::{
    HashSet,
    HashMap
};

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
    },
    trace::Traces
};

pub(crate) struct Machine {
    name: Ident,
    states: StateMems,
    inits: Inits,
    traces: Traces,
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

        let Traces {
            traces: trace_states
        } = machine_blk.parse()?;
        let mut trace_states: HashMap<State, Vec<Ident>> = trace_states.into();

        let mut mem_defs: Vec<MemDef> = mem_defs.into();

        while !machine_blk.is_empty() {
            let EDefinition {origs: event_origs, dests: event_dests, event} = machine_blk.parse()?;

            if let Some(first) = events.get(&event) {
                let mut err = Error::new_spanned(&event.name, format!{"Duplicate event: {}", event.name});
                err.combine(Error::new_spanned(&first.name, format!{"First declared here"}));
                return Err(err);
            }

            for (_, events) in trace_states.iter_mut().filter(|(key, _)| event_origs.contains(key)) {
                events.push(event.name.clone());
            }

            events.insert(event);
            states.extend(event_origs);
            states.extend(event_dests);
        }

        mem_defs.extend(states.difference(&mem_states.into()).cloned().map(|s| s.into()));

        let state_mems: Vec<StateMem> = mem_defs.iter().cloned().map(|md| md.into()).collect();

        Ok (
            Machine {
                name,
                inits,
                traces: trace_states.into(),
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
        let traces = &self.traces;
        let states = &self.states;
        let events = &self.events;
        let variants = &self.variants;

        tokens.extend(quote! {
            #[allow(non_snake_case)]
            mod #name {
                pub use static_fsm::{Transition, Init, ToEnum, ToMemEnum};

                use core::marker::PhantomData;
                use static_fsm::{Event, State, SM, EntryPoint};

                #[derive(Clone)]
                pub struct FSM<S: State> {
                    _s: PhantomData<S>,
                }

                impl<S: State> SM for FSM<S> {
                    type State = S;
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
                #traces
                pub enum Variants {
                    #variants
                }
            }
        });
    }
}
