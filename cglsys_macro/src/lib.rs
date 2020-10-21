extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    custom_punctuation, parenthesized,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    token::{Comma, FatArrow},
    Expr, Ident, Token,
};

struct Module {
    pub is_expr: bool,
    pub letter: Ident,
    pub parameters: Option<Punctuated<Expr, Comma>>,
}

impl Parse for Module {
    fn parse(input: ParseStream) -> Result<Self> {
        let letter = input.parse()?;

        let parameters = if input.peek(syn::token::Paren) {
            let inner;
            parenthesized!(inner in input);
            Some(inner.parse_terminated(Expr::parse)?)
        } else {
            None
        };

        // This module is a replace expression if it's parameters aren't all paths of length 1
        let mut is_expr = false;

        // TODO: Ideally if we knew the module was on the left or right hand side we would not have to do
        // this. We would just know whether to look for Ident or Expr. But I can't figure out how I
        // would pass that information without writing a separate Module struct. Maybe like an
        // enum, Module::Pattern and Module::Expr or something.
        if let Some(parameters) = &parameters {
            parameters.iter().for_each(|expr| match expr {
                syn::Expr::Path(path) => {
                    if let None = path.path.get_ident() {
                        is_expr = true;
                    }
                }
                _ => {
                    is_expr = true;
                }
            });
        }

        Ok(Module {
            letter,
            is_expr,
            parameters,
        })
    }
}

impl ToTokens for Module {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let letter = &self.letter;
        let parameters = self.parameters.iter();

        if parameters.len() == 0 {
            tokens.extend(quote! {
                Self::Alphabet::#letter
            });
        } else {
            tokens.extend(quote! {
                 Self::Alphabet::#letter(#(#parameters),*)
            });
        }
    }
}

#[derive(Default)]
struct String {
    pub string: Vec<Module>,
    pub is_expr: bool,
}

impl Parse for String {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut string = vec![];
        let mut is_expr = false;

        while let Ok(module) = input.parse::<Module>() {
            if module.is_expr {
                is_expr = true;
            }

            string.push(module);
        }

        Ok(String { is_expr, string })
    }
}

impl ToTokens for String {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let modules = self.string.iter();
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
        } else if modules.string.len() == 1 {
            left_context = String::default();
            module = modules.string.pop().unwrap();
        } else {
            return Err(input.error("Expected <"));
        }

        if let Ok(_) = input.parse::<Token![>]>() {
            right_context = input.parse()?;
        } else {
            right_context = String::default();
        }

        //if left_context.is_expr || right_context.is_expr {
        //return Err(input.error("Expected parameters, not expressions"));
        //}

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

        tokens.extend(
            match (left_context.string.len(), right_context.string.len()) {
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
                },
            },
        );
    }
}

struct ProductionRule {
    pub pattern: ProductionRulePattern,
    pub replace: String,
}

impl Parse for ProductionRule {
    fn parse(input: ParseStream) -> Result<Self> {
        let pattern: ProductionRulePattern = input.parse()?;
        input.parse::<FatArrow>()?;
        let replace: String = input.parse()?;

        Ok(ProductionRule { pattern, replace })
    }
}

impl ToTokens for ProductionRule {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ProductionRule { pattern, replace } = self;

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
                match (left_context, *module, right_context) {
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
