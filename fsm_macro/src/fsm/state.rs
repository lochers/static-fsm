use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    Ident,
};

use std::hash::{
    Hasher,
    Hash
};

use std::collections::HashSet;

pub(crate) struct States {
    states: HashSet<State>
}

impl Into<HashSet<State>> for States {
    fn into(self) -> HashSet<State> {
        self.states
    }
}

impl From<HashSet<State>> for States {
    fn from(states: HashSet<State>) -> Self {
        Self {
            states
        }
    }
}

impl IntoIterator for States {
    type Item = State;
    type IntoIter = std::collections::hash_set::IntoIter<State>;

    fn into_iter(self) -> Self::IntoIter {
        self.states.into_iter()
    }
}

#[derive(Clone)]
pub(crate) struct State {
    pub name: Ident
}

impl Parse for State {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name: Ident = input.parse()?;

        Ok (Self { name } )
    }
}

impl ToTokens for States {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.states.iter().for_each(|state| state.to_tokens(tokens));
    }
}

impl ToTokens for State {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;

        tokens.extend(quote! {
            #[derive(Clone, Copy, PartialEq, Eq)]
            pub struct #name;
            impl State for #name {}
        });
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for State { }

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}




