use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    token::Comma,
    Token,
    braced,
    Ident,
    Type,
    Error,
};

use std::slice::Iter;

use std::collections::HashSet;

use crate::fsm::state::{
    States,
    State
};

pub(crate) struct MemDefs {
    mem_defs: Vec<MemDef>
}

#[derive(Clone)]
pub(crate) struct MemDef {
    pub state: Ident,
    pub memory: Option<Type>
}

pub(crate) struct MemDefBlk {
    pub states: States,
    pub mem_defs: MemDefs
}

pub(crate) struct StateMem {
    pub state: Ident,
    pub memory: Option<Type>
}

pub(crate) struct StateMems {
    state_mems: Vec<StateMem>
}

impl From<State> for MemDef {
    fn from(state: State) -> Self {
        Self {
            state: state.name,
            memory: None
        }
    }
}

impl From<MemDef> for StateMem {
    fn from(mem_def: MemDef) -> Self {
        Self {
            state: mem_def.state,
            memory: mem_def.memory
        }
    }
}

impl IntoIterator for MemDefs {
    type Item = MemDef;
    type IntoIter = std::vec::IntoIter<MemDef>;

    fn into_iter(self) -> Self::IntoIter {
        self.mem_defs.into_iter()
    }
}

impl From<Vec<MemDef>> for MemDefs {
    fn from(mem_defs: Vec<MemDef>) -> Self {
        Self {
            mem_defs
        }
    }
}

impl From<Vec<StateMem>> for StateMems {
    fn from(state_mems: Vec<StateMem>) -> Self {
        Self {
            state_mems
        }
    }
}
 
impl From<MemDefs> for Vec<MemDef> {
    fn from(mem_defs: MemDefs) -> Self {
        mem_defs.mem_defs
    }
}

impl Parse for MemDefBlk {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut states: HashSet<State> = HashSet::new();
        let mut mem_defs: Vec<MemDef> = Vec::new();

        let fork = input.fork();
        let state_storage: Ident = fork.parse()?;
        if state_storage == "Memory" {
            let _: Ident = input.parse()?;
            let storage_blk;
            braced!(storage_blk in input);

            while !storage_blk.is_empty() {
                let mut def_states: Vec<Ident> = Vec::new();

                loop {
                    let state: State = storage_blk.parse()?;
                    if let Some(first) = states.get(&state) {
                        let mut err = Error::new_spanned(&state.name, format!{"Duplicate transition origin: {}", state.name});
                        err.combine(Error::new_spanned(&first.name, format!{"First declared here"}));

                        return Err(err);
                    }

                    states.insert(state.clone());
                    def_states.push(state.name);

                    if storage_blk.peek(Token![,]) {
                        let _: Comma = storage_blk.parse()?;
                    } else {
                        break;
                    }
                }

                let _: Token![=>] = storage_blk.parse()?;

                let memory: Type = storage_blk.parse()?;

                def_states.into_iter().for_each(|def_state| mem_defs.push(MemDef::new(def_state, Some(memory.clone()))));
                
                if storage_blk.is_empty() {
                    break;
                }

                let _: Comma = storage_blk.parse()?;
            }
        }

        return Ok ( MemDefBlk {
            states: states.into(),
            mem_defs: mem_defs.into()
        });    
    }
}

impl MemDef {
    fn new(state:Ident, memory: Option<Type>) -> Self {
        Self {
            state,
            memory
        }
    }
}

impl MemDefs {
    #[inline]
    fn iter(&self) -> Iter<'_, MemDef> {
        self.mem_defs.iter()
    }
}

impl StateMems {
    #[inline]
    fn iter(&self) -> Iter<'_, StateMem> {
        self.state_mems.iter()
    }
}

impl ToTokens for StateMems {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.iter().for_each(|state_mem| state_mem.to_tokens(tokens));
    }
}

impl ToTokens for StateMem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let state = &self.state;
        if let Some(memory) = &self.memory {
            tokens.extend(quote! {
                #[derive(Clone, Copy, PartialEq, Eq)]
                pub struct #state;
                impl State for #state {}

                impl ToMemEnum for FSM<#state> {
                    type Repr = Variants;
                    type Mem = #memory;

                    fn to_enum(self, memory: #memory) -> Self::Repr {
                        Variants::#state(self, memory)
                    }
                }

            });
        } else {
            tokens.extend(quote! {
                #[derive(Clone, Copy, PartialEq, Eq)]
                pub struct #state;
                impl State for #state {}

                impl ToEnum for FSM<#state> {
                    type Repr = Variants;
                    fn to_enum(self) -> Self::Repr {
                        Variants::#state(self)
                    }
                }
            });
        }
    }
}

impl ToTokens for MemDefs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.iter().for_each(|mem_def| mem_def.to_tokens(tokens));
    }
}

impl ToTokens for MemDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let state = &self.state;

        if let Some(memory) = &self.memory {
            tokens.extend(quote! {
                #state(FSM<#state>,#memory),
            });
        } else {
            tokens.extend(quote! {
                #state(FSM<#state>),
            });
        }
    }
}
