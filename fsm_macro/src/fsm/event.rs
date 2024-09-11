use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    token::Comma,
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

use crate::fsm::variant::{
    EventVariants,
    EventVariant
};

use crate::fsm::transition::{
    TDefinition,
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
    pub variants: EventVariants
}

impl Parse for EDefinition {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut prevs: HashSet<State> = HashSet::new();
        let mut nexts: HashSet<State> = HashSet::new();
        let mut inits: HashSet<State> = HashSet::new();

        let mut transitions: Vec<Transition> = Vec::new();
        let mut variants: HashSet<EventVariant> = HashSet::new();

        let event: Ident = input.parse()?;

        let event_blk;
        braced!(event_blk in input);

        while !event_blk.is_empty() {
            let TDefinition {
                origins: t_prevs, 
                destination: t_next
            } = event_blk.parse()?;

            if let Some(t_prevs) = t_prevs {

                let t_prevs: HashSet<State> = t_prevs.into();

                for i in prevs.intersection(&t_prevs) {
                    let first = prevs.get(&i).unwrap();
                    let current = t_prevs.get(&i).unwrap();

                    let mut err = Error::new_spanned(&current.name, format!{"Duplicate transition origin: {}", current.name});
                    err.combine(Error::new_spanned(&first.name, format!{"First declared here"}));

                    return Err(err);
                }

                nexts.insert(t_next.clone());
                prevs.extend(t_prevs.iter().cloned());

                transitions.extend(Transitions::generate(Some(t_prevs), t_next.clone(), event.clone()));

            } else {
                if let Some(first) = inits.get(&t_next) {
                    let mut err = Error::new_spanned(&t_next.name, format!{"Duplicate sate initialisation: {}", t_next.name});
                    err.combine(Error::new_spanned(&first.name, format!{"First declared here"}));

                    return Err(err);
                }

                inits.insert(t_next.clone());
                transitions.extend(Transitions::generate(t_prevs, t_next.clone(), event.clone()));
            }

            variants.insert(EventVariant::new(t_next.name, event.clone()));

            if event_blk.is_empty() {
                break;
            }

            let _: Comma = event_blk.parse()?;
        }
        
        nexts.extend(inits);
        prevs.extend(nexts);

        Ok ( EDefinition {
            states: prevs.into(),
            event: Event { 
                name: event, 
                transitions: transitions.into(),
                variants: variants.into()
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
        let variants = &self.variants;

        tokens.extend(quote! {
            #[derive(Clone, Copy, PartialEq, Eq)]
            pub struct #name;
            impl Event for #name {}

            #transitions
            #variants
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




