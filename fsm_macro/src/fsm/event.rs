use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    token::Comma,
    Token,
    Ident,
    Error
};

use std::hash::{
    Hasher,
    Hash
};

use std::collections::{
    hash_set::Iter,
    HashSet
};

use crate::fsm::transition::{
    Transitions,
    Transition
};
use crate::fsm::state::{
    States,
    State
};

pub(crate) struct EDefinition {
    pub states: States,
    pub event: Event
}

pub(crate) struct Events {
    events: HashSet<Event>
}

pub(crate) struct Event {
    pub name: Ident,
    pub transitions: Transitions,
}

impl Parse for EDefinition {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut orig_states: HashSet<State> = HashSet::new();
        let mut dest_states: HashSet<State> = HashSet::new();
        let mut transitions: Vec<Transition> = Vec::new();

        let event: Ident = input.parse()?;

        let event_blk;
        braced!(event_blk in input);

        while !event_blk.is_empty() {
            let mut def_states: Vec<Ident> = Vec::new();

            loop {
                let state: State = event_blk.parse()?;
                if let Some(first) = orig_states.get(&state) {
                    let mut err = Error::new_spanned(&state.name, format!{"Duplicate transition origin: {}", state.name});
                    err.combine(Error::new_spanned(&first.name, format!{"First declared here"}));

                    return Err(err);
                }

                orig_states.insert(state.clone());
                def_states.push(state.name);

                if event_blk.peek(Token![,]) {
                    let _: Comma = event_blk.parse()?;
                } else {
                    break;
                }
            }

            let _: Token![=>] = event_blk.parse()?;

            let dest_state: State = event_blk.parse()?;
            dest_states.insert(dest_state.clone());

            transitions.extend(Transitions::generate(def_states, dest_state.name, event.clone()));
            
            if event_blk.is_empty() {
                break;
            }

            let _: Comma = event_blk.parse()?;
        }
        
        orig_states.extend(dest_states);

        Ok ( EDefinition {
            states: orig_states.into(),
            event: Event { 
                name: event, 
                transitions: transitions.into(),
            }
        } )
    }
}

impl Into<HashSet<Event>> for Events {
    fn into(self) -> HashSet<Event> {
        self.events
    }
}

impl From<HashSet<Event>> for Events {
    fn from(events: HashSet<Event>) -> Self {
        Self {
            events
        }
    }
}

impl Events {
    #[inline]
    fn iter(&self) -> Iter<'_, Event> {
        self.events.iter()
    }
}

impl ToTokens for Events {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.iter().for_each(|event| event.to_tokens(tokens));
    }
}

impl ToTokens for Event {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let transitions = &self.transitions;

        tokens.extend(quote! {
            #[derive(Clone, Copy, PartialEq, Eq)]
            pub struct #name;
            impl Event for #name {}

            #transitions
        });
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Event { }

impl Hash for Event {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}




