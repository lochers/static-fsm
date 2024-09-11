
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    token::Comma,
    Token,
    Ident,
    Error,
};

use std::slice::Iter;

use std::collections::HashSet;

use crate::fsm::state::{
    States,
    State
};

pub(crate) struct Transitions {
    transitions: Vec<Transition>
}

impl IntoIterator for Transitions {
    type Item = Transition;
    type IntoIter = std::vec::IntoIter<Transition>;

    fn into_iter(self) -> Self::IntoIter {
        self.transitions.into_iter()
    }
}

impl Into<Vec<Transition>> for Transitions {
    fn into(self) -> Vec<Transition> {
        self.transitions
    }
}

impl From<Vec<Transition>> for Transitions {
    fn from(transitions: Vec<Transition>) -> Self {
        Self {
            transitions
        }
    }
}

pub(crate) struct TDefinition {
    pub origins: Option<States>,
    pub destination: State
}

pub(crate) struct Transition {
    pub event: Ident,
    pub next: Ident,
    pub prev: Option<Ident>
}

impl Parse for TDefinition {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(Token![=>]) {
            let _: Token![=>] = input.parse()?;

            let next: State = input.parse()?;

            Ok( TDefinition {
                origins: None,
                destination: next
            })
        } else {
            let mut prevs: HashSet<State> = HashSet::new(); 

            prevs.insert(input.parse()?);

            loop {
                if input.peek(Token![,]) {
                    let _: Comma = input.parse()?;
                    
                    let state: State = State::parse(&input)?;

                    if let Some(first) = prevs.get(&state) {
                        let mut err = Error::new_spanned(&state.name, format!{"Duplicate transition origin: {}", state.name});
                        err.combine(Error::new_spanned(&first.name, format!{"First declared here"}));

                        return Err(err);
                    }

                    prevs.insert(state);
                } else {
                    break;
                }
            }

            let _: Token![=>] = input.parse()?;

            let next: State = input.parse()?;

            if let Some(first) = prevs.get(&next) {
                let mut err = Error::new_spanned(&next.name, format!{"Destination state can not be an origin to itself: {}", next.name});
                err.combine(Error::new_spanned(&first.name, format!{"First declared here"}));

                return Err(err);
            }

            Ok( TDefinition {
                origins: Some( prevs.into() ),
                destination: next
            })
        }
    }
}

impl Transitions {
    #[inline]
    fn iter(&self) -> Iter<'_, Transition> {
        self.transitions.iter()
    }

    pub fn generate<I>(prevs: Option<I>, next: State, event: Ident) -> Self
    where
        I: IntoIterator<Item = State>
    {
        if let Some(prevs) = prevs {
            Self {
                transitions: prevs.into_iter()
                    .map(|prev| 
                         Transition {
                             event: event.clone(), 
                             next: next.name.clone(), 
                             prev: Some( prev.name )
                         }).collect()
            }
        } else {
            Self {
                transitions: vec![Transition {
                    event,
                    next: next.name,
                    prev: None
                }]
            }

        }
    }
}

impl ToTokens for Transitions {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.iter().for_each(|transition| transition.to_tokens(tokens));
    }
}

impl ToTokens for Transition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let event = &self.event;
        let next = &self.next;
        
        if let Some(prev) = &self.prev {
            let prev = &prev;
            tokens.extend(quote! {
                impl<E: Event> Transition<#event> for FSM<#prev, E> {
                    type SM = FSM<#next, #event>;

                    fn t(self, event: #event) -> Self::SM {
                        FSM {
                            s: #next,
                            e: event 
                        }
                    }
                }
            });
        } else {
            tokens.extend(quote! {
                impl EntryPoint<#next> for #event {
                    type Event = #event;
                    type SM = FSM<#next, #event>;

                    fn fsm() -> Self::SM {
                        FSM {
                            s: #next,
                            e: #event
                        }
                    }
                }
            });

        }


    }
}
