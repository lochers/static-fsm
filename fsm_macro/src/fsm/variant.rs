
use std::hash::{
    Hash,
    Hasher
};
use proc_macro2::{
    TokenStream,
    Span
};
use quote::{quote, ToTokens};
use syn::Ident;

use std::collections::HashSet;

#[derive(Clone)]
pub(crate) struct EventVariant {
    pub name: Ident,
    pub event: Ident,
    pub state: Ident
}

pub(crate) struct EventVariants {
    variants: HashSet<EventVariant>
}

impl EventVariant {
    pub fn new(state: Ident, event: Ident) -> Self {
        let name = Ident::new(
            &format!("{}By{}", &state, &event),
            Span::call_site()
        );
        Self {
            name,
            state,
            event
        }
        
    }
}

impl IntoIterator for EventVariants {
    type Item = EventVariant;
    type IntoIter = std::collections::hash_set::IntoIter<EventVariant>;

    fn into_iter(self) -> Self::IntoIter {
        self.variants.into_iter()
    }
}

impl Into<HashSet<EventVariant>> for EventVariants {
    fn into(self) -> HashSet<EventVariant> {
        self.variants
    }
}

impl From<HashSet<EventVariant>> for EventVariants {
    fn from(variants: HashSet<EventVariant>) -> Self {
        Self {
            variants
        }
    }
}

impl Hash for EventVariant {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
    
}

impl PartialEq for EventVariant {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for EventVariant { }

impl EventVariants {
    pub fn iter(&self) -> std::collections::hash_set::Iter<EventVariant> {
        self.variants.iter()
    }
}

pub(crate) struct Variant {
    pub name: Ident,
    pub event: Ident,
    pub state: Ident
}

pub(crate) struct Variants {
    variants: Vec<Variant>
}

impl From<EventVariant> for Variant {
    fn from(variant: EventVariant) -> Self {
        Self {
            name: variant.name,
            state: variant.state,
            event: variant.event,
        }
    }
}

impl Into<Vec<Variant>> for Variants {
    fn into(self) -> Vec<Variant> {
        self.variants
    }
}

impl From<Vec<Variant>> for Variants {
    fn from(variants: Vec<Variant>) -> Self {
        Self {
            variants
        }
    }
}

impl ToTokens for EventVariants {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.iter().for_each(|variant| variant.to_tokens(tokens));
    }
}

impl ToTokens for EventVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let event = &self.event;
        let state = &self.state;
        let name = &self.name;

        
        tokens.extend(quote! {
            impl ToEnum for FSM<#state, #event> {
                type Repr = Variants;      

                fn to_enum(self) -> Self::Repr {
                    Variants::#name(self)
                }
            }
        });
    }
}

impl ToTokens for Variant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let state = &self.state;
        let event = &self.event;

        tokens.extend(quote! {
            #name(FSM<#state, #event>)
        });
    }
}

impl ToTokens for Variants {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let variants = &self.variants;

        tokens.extend(quote! {
            pub enum Variants {
                #(#variants),*
            }
        })
    }
}
