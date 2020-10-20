extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    custom_punctuation, parenthesized,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, Ident, Token,
};

struct Module {
    pub letter: Ident,
    pub parameters: Punctuated<Expr, Token![,]>,
}

impl Parse for Module {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Module {
            letter: input.parse()?,
            parameters: {
                let inner;
                parenthesized!(inner in input);
                inner.parse_terminated(Expr::parse)?
            },
        })
    }
}

impl ToTokens for Module {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let letter = &self.letter;
        let parameters = self.parameters.iter();
        tokens.extend(quote! {
             Self::Alphabet::#letter(#(#parameters),*)
        });
    }
}

struct String(Vec<Module>);

impl Parse for String {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut string = String(vec![]);

        while let Ok(module) = input.parse::<Module>() {
            string.0.push(module);
        }

        Ok(string)
    }
}

impl ToTokens for String {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let modules = self.0.iter();
        tokens.extend(quote! {
            #(#modules),*
        });
    }
}

struct ProductionRulePattern {
    pub left_context: String,
    pub right_context: String,
    pub module: Module,
}

impl Parse for ProductionRulePattern {
    fn parse(input: ParseStream) -> Result<Self> {
        let left_context: String;
        let module: Module;
        let right_context: String;

        let mut modules: String = input.parse()?;
        if let Ok(_) = input.parse::<Token![<]>() {
            left_context = modules;
            module = input.parse()?;
        } else if modules.0.len() == 1 {
            left_context = String(vec![]);
            module = modules.0.pop().unwrap();
        } else {
            return Err(input.error("expected <"));
        }

        if let Ok(_) = input.parse::<Token![>]>() {
            right_context = input.parse()?;
        } else {
            right_context = String(vec![]);
        }

        Ok(ProductionRulePattern {
            left_context,
            module,
            right_context,
        })
    }
}

impl ToTokens for ProductionRulePattern {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let left_context = &self.left_context;
        let right_context = &self.right_context;
        let module = &self.module;

        tokens.extend(match (left_context.0.len(), right_context.0.len()) {
            (a, b) if a > 0 && b > 0 => quote! {
                (&[.., #left_context], #module, &[#right_context, ..])
            },
            (a, b) if a == 0 && b > 0 => quote! {
                (_, #module, &[#right_context, ..])
            },
            (a, b) if a > 0 && b == 0 => quote! {
                (&[.., #left_context], #module, _)
            },
            _ => quote! {
                (_, #module, _)
            }
        });
    }
}

struct ProductionRule {
    pub pattern: ProductionRulePattern,
    pub replace: String,
}

impl Parse for ProductionRule {
    fn parse(input: ParseStream) -> Result<Self> {
        custom_punctuation!(RightArrow, =>);

        let pattern: ProductionRulePattern = input.parse()?;
        input.parse::<RightArrow>()?;
        let replace: String = input.parse()?;

        Ok(ProductionRule { pattern, replace })
    }
}

impl ToTokens for ProductionRule {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let pattern = &self.pattern;
        let replace = &self.replace;

        tokens.extend(quote! {
            #pattern => vec![#replace]
        });
    }
}

struct ProductionRules(Punctuated<ProductionRule, Token![,]>);

impl Parse for ProductionRules {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ProductionRules(
            input.parse_terminated(ProductionRule::parse)?,
        ))
    }
}

impl ToTokens for ProductionRules {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let rules = self.0.iter();
        tokens.extend(quote! {
            fn production_rules(&self, module: &Self::Alphabet, left_context: &[Self::Alphabet], right_context: &[Self::Alphabet]) -> Vec<Self::Alphabet> {
                match (left_context, module, right_context) {
                    #(#rules),*,
                    (_, m, _) => vec![m.clone()],
                }
            }
        });
    }
}

#[proc_macro]
pub fn production_rules(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let rules: ProductionRules = parse_macro_input!(item);
    quote!(#rules).into()
}
