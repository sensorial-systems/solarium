use anyhow::{bail, Result};
use sha2::{Digest, Sha256};

/// The name of the attribute that configures one instruction of a `#[program]`.
const ATTRIBUTE: &str = "instruction";

/// What a method's `#[instruction(...)]` says about how it is called.
///
/// ```ignore
/// #[instruction(discriminator = 16)]
/// pub fn request_bet(...) -> Result<()>
///
/// #[instruction(discriminator = 3, alias = "global:process_undelegation")]
/// pub fn undelegate(...) -> Result<()>
/// ```
///
/// A value is either an integer, read as the little-endian `u64` the instruction starts with, or a
/// string, hashed the way every default is. Without the attribute a method answers to
/// `"global:<name>"`, as it always has.
pub struct InstructionAttribute {
    /// The one a client sends.
    pub discriminator: u64,
    /// Others the method also answers to, for callers that cannot be changed — a program that
    /// calls back with a tag of its own choosing.
    pub aliases: Vec<u64>,
}

impl InstructionAttribute {
    pub fn parse(method: &syn::ImplItemFn) -> Result<Self> {
        let mut discriminator = None;
        let mut aliases = Vec::new();
        for attribute in method.attrs.iter().filter(|a| a.path().is_ident(ATTRIBUTE)) {
            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("discriminator") {
                    if discriminator.is_some() {
                        return Err(meta.error("the discriminator is given twice"));
                    }
                    discriminator = Some(value(&meta.value()?.parse()?)?);
                    Ok(())
                } else if meta.path.is_ident("alias") {
                    aliases.push(value(&meta.value()?.parse()?)?);
                    Ok(())
                } else {
                    Err(meta.error("expected `discriminator` or `alias`"))
                }
            })?;
        }
        let discriminator =
            discriminator.unwrap_or_else(|| hashed(&format!("global:{}", method.sig.ident)));
        Ok(Self {
            discriminator,
            aliases,
        })
    }

    /// Every discriminator the method answers to, its own first.
    pub fn all(&self) -> impl Iterator<Item = u64> + '_ {
        std::iter::once(self.discriminator).chain(self.aliases.iter().copied())
    }

    /// Takes the attribute off, since nothing else knows what it means.
    pub fn strip(method: &mut syn::ImplItemFn) {
        method.attrs.retain(|a| !a.path().is_ident(ATTRIBUTE));
    }
}

fn value(literal: &syn::Lit) -> syn::Result<u64> {
    match literal {
        syn::Lit::Int(integer) => integer.base10_parse(),
        syn::Lit::Str(namespace) => Ok(hashed(&namespace.value())),
        _ => Err(syn::Error::new_spanned(
            literal,
            "expected an integer or a string to hash",
        )),
    }
}

fn hashed(namespace: &str) -> u64 {
    let hash = Sha256::digest(namespace.as_bytes());
    u64::from_le_bytes(hash[..8].try_into().unwrap())
}

/// Refuses two methods answering to the same discriminator: one of them would never be reached,
/// and nothing about the call would say which.
pub fn check_unique<'a>(methods: impl IntoIterator<Item = (&'a syn::Ident, u64)>) -> Result<()> {
    let mut seen = std::collections::HashMap::new();
    for (method, discriminator) in methods {
        if let Some(other) = seen.insert(discriminator, method) {
            bail!("`{other}` and `{method}` both answer to discriminator {discriminator}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attribute(method: syn::ImplItemFn) -> InstructionAttribute {
        InstructionAttribute::parse(&method).unwrap()
    }

    #[test]
    fn a_method_without_the_attribute_keeps_its_hashed_name() {
        let plain = attribute(syn::parse_quote! { pub fn resolve_bet(&self) {} });
        assert_eq!(plain.discriminator, hashed("global:resolve_bet"));
        assert!(plain.aliases.is_empty());
    }

    #[test]
    fn numbers_and_names_both_give_a_discriminator() {
        let undelegate = attribute(syn::parse_quote! {
            #[instruction(discriminator = 3, alias = "global:process_undelegation")]
            pub fn undelegate(&self) {}
        });
        assert_eq!(undelegate.discriminator, 3);
        // The delegation program's fixed callback tag.
        assert_eq!(
            undelegate.aliases,
            [u64::from_le_bytes([196, 28, 41, 206, 48, 37, 51, 167])]
        );
    }

    #[test]
    fn anything_else_in_the_attribute_is_refused() {
        let method: syn::ImplItemFn = syn::parse_quote! {
            #[instruction(number = 3)]
            pub fn undelegate(&self) {}
        };
        assert!(InstructionAttribute::parse(&method).is_err());
    }

    #[test]
    fn two_methods_on_one_discriminator_are_refused() {
        let a: syn::Ident = syn::parse_quote!(bet);
        let b: syn::Ident = syn::parse_quote!(hold);
        assert!(check_unique([(&a, 16), (&b, 20)]).is_ok());
        let error = check_unique([(&a, 16), (&b, 16)]).unwrap_err();
        assert!(error.to_string().contains("`bet` and `hold`"));
    }
}
