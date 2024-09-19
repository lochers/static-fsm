use proc_macro2::{
    TokenStream,
    Span
};

use quote::{quote, ToTokens};
use syn::{
    bracketed,
    parse::{Parse, ParseStream, Result},
    token::Comma,
    Ident,
    Error
};

use std::collections::{
    HashSet,
    HashMap,
    hash_map::Iter,
};

use crate::fsm::state::State;

pub(crate) struct Traces {
    pub traces: HashMap<State, Vec<Ident>>
}

impl Traces {
    #[inline]
    pub(crate) fn iter(&self) -> Iter<'_, State, Vec<Ident>> {
        self.traces.iter()
    }
}

impl Into<HashMap<State, Vec<Ident>>> for Traces {
    fn into(self) -> HashMap<State, Vec<Ident>> {
        self.traces
    }
}

impl From<HashMap<State, Vec<Ident>>> for Traces {
    fn from(traces: HashMap<State, Vec<Ident>>) -> Self {
        Self {
            traces
        }
    }
}

impl IntoIterator for Traces {
    type Item = (State, Vec<Ident>);
    type IntoIter = std::collections::hash_map::IntoIter<State, Vec<Ident>>;

    fn into_iter(self) -> Self::IntoIter {
        self.traces.into_iter()
    }
}


impl Parse for Traces {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut trace_ref: HashSet<State> = HashSet::new();
        let mut traces: HashMap<State, Vec<Ident>> = HashMap::new();

        let fork = input.fork();
        if let Ok (memory) = fork.parse::<Ident>() {
            if memory == "Trace" {
                let _:Ident = input.parse()?;

                let trace_blk;
                bracketed!(trace_blk in input);

                while !trace_blk.is_empty() {
                    let state: State = trace_blk.parse()?;
                    
                    if let Some(first_trace) = trace_ref.get(&state) {
                        let mut err = Error::new_spanned(&state.name, format!{"Duplicate sate traceialisation: {}", state.name});
                        err.combine(Error::new_spanned(&first_trace.name, format!{"First declared here"}));

                        return Err(err);
                    }

                    trace_ref.insert(state.clone());

                    traces.insert(state, Vec::new());

                    if trace_blk.is_empty() {
                        break;
                    }

                    let _: Comma = trace_blk.parse()?;
                }
            }
        }

        Ok ( traces.into() )
    }
}

impl ToTokens for Traces {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for (state, events) in self.iter() {

            let name = Ident::new(&format!("{}Trace", &state.name), Span::call_site());

            tokens.extend(quote! {
                pub enum #name {
                    #(#events(#events)),*
                }
            });
        }
    }
}

