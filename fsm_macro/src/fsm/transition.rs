
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Ident;

use std::slice::Iter;

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

pub(crate) struct Transition {
    pub event: Ident,
    pub next: Ident,
    pub prev: Ident
}

impl Transitions {
    #[inline]
    fn iter(&self) -> Iter<'_, Transition> {
        self.transitions.iter()
    }

    pub fn generate<I>(prevs: I, next: Ident, event: Ident) -> Self
    where
        I: IntoIterator<Item = Ident>
    {

        Self {
            transitions: prevs.into_iter()
                .map(|prev| 
                     Transition {
                         event: event.clone(), 
                         next: next.clone(), 
                         prev
                     }).collect()
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
        let prev = &self.prev;
        
        tokens.extend(quote! {
            impl Transition<#event> for FSM<#prev> {
                type SM = FSM<#next>;

                fn t(self, _e: #event) -> Self::SM {
                    FSM {
                        _s: PhantomData
                    }
                }
            }
        });
    }
}
