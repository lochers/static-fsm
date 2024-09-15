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

use crate::fsm::state::State;

pub(crate) struct Inits {
    inits: HashSet<Init>
}

impl Inits {
    #[inline]
    pub(crate) fn iter(&self) -> Iter<'_, Init> {
        self.inits.iter()
    }
}

impl Into<HashSet<Init>> for Inits {
    fn into(self) -> HashSet<Init> {
        self.inits
    }
}

impl From<HashSet<Init>> for Inits {
    fn from(inits: HashSet<Init>) -> Self {
        Self {
            inits
        }
    }
}

impl IntoIterator for Inits {
    type Item = Init;
    type IntoIter = std::collections::hash_set::IntoIter<Init>;

    fn into_iter(self) -> Self::IntoIter {
        self.inits.into_iter()
    }
}

#[derive(Clone)]
pub(crate) struct Init {
    pub state: Ident
}

impl Parse for Inits {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut inits: HashSet<Init> = HashSet::new();

        let init: Ident = input.parse()?;
        if init != "Init" {
            let err = Error::new_spanned(&init, format!{"Expected Init block, got: {}", init});
            return Err(err);
        }

        let init_blk;
        braced!(init_blk in input);

        while !init_blk.is_empty() {
            let init_state: Init = init_blk.parse()?;
            
            if let Some(first_init) = inits.get(&init_state) {
                let mut err = Error::new_spanned(&init_state.state, format!{"Duplicate sate initialisation: {}", init_state.state});
                err.combine(Error::new_spanned(&first_init.state, format!{"First declared here"}));

                return Err(err);
            }

            inits.insert(init_state);

            if init_blk.is_empty() {
                break;
            }

            let _: Comma = init_blk.parse()?;
        }

        Ok ( Inits {
            inits: inits.into()
        } )
    }
}

impl From<Init> for State {
    fn from(init: Init) -> Self {
        Self {
            name: init.state
        }
    }
}

impl Parse for Init {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok ( Init {
            state: input.parse()?
        })
    }
}

impl ToTokens for Inits {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.iter().for_each(|init| init.to_tokens(tokens));
    }
}

impl ToTokens for Init {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let state = &self.state;

        tokens.extend(quote! {
            impl EntryPoint for #state { }
        });
    }
}

impl PartialEq for Init {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

impl Eq for Init { }

impl Hash for Init {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.state.hash(state)
    }
}
