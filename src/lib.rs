#![doc = include_str!("../README.md")]
#![allow(clippy::needless_doctest_main)]

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::mem;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::{
    parse_quote, visit, visit_mut, AttrStyle, Error, FnArg, GenericParam, Generics, ItemFn,
    Lifetime, Meta, PredicateLifetime, PredicateType, Receiver, Result, Type, TypeArray, TypeParam,
    TypePath, Visibility, WhereClause, WherePredicate,
};

/// Converts a regular function into an extension function.
///
/// # Examples
/// ```
#[doc = include_str!("../examples/demo.rs")]
/// ```
///
/// <details>
/// <summary>All supported function signatures</summary>
///
/// ```
#[doc = include_str!("../tests/signatures.rs")]
/// ```
/// </details>
#[proc_macro_attribute]
pub fn extfn(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(attr.into(), input.into())
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    if !attr.is_empty() {
        return Err(Error::new(
            Span::call_site(),
            "attribute arguments are not allowed",
        ));
    }

    let mut function = syn::parse2::<ItemFn>(input)?;

    let first_param = function.sig.inputs.first_mut().ok_or(Error::new(
        function.sig.paren_token.span.span(),
        "function must have a parameter named `self`",
    ))?;

    let self_param = match first_param {
        FnArg::Receiver(receiver) => receiver,
        FnArg::Typed(typed) => {
            return Err(Error::new(
                typed.pat.span(),
                "parameter must be called `self`",
            ));
        }
    };

    if self_param.reference.is_some() || self_param.colon_token.is_none() {
        return Err(Error::new(
            self_param.span(),
            "the `self` parameter must have a type",
        ));
    }

    // take the type `T` from `self: T` or `self: &T`
    let mut dest_type = self_param.ty.as_mut();
    while let Type::Paren(paren) = dest_type {
        dest_type = paren.elem.as_mut(); // skip parenthesis
    }
    if let Type::Reference(reference) = dest_type {
        dest_type = reference.elem.as_mut(); // skip reference
    };
    let mut self_type = mem::replace(dest_type, parse_quote!(Self));

    simplify_self_param(self_param);

    let mut generics = Generics::default();

    // convert `impl Trait` into `T: Trait`
    ImplTraitsIntoGenerics::new(&mut generics).visit_type_mut(&mut self_type);

    // extract all generics used in `self_type`
    let extracted_generics = extract_impl_generics(&self_type, &mut function.sig.generics);
    generics.params.extend(extracted_generics.params);

    move_bounds_to_where_clause(
        &mut generics.params,
        &mut function.sig.generics.where_clause,
    );

    // strip extra parenthesis that might've had a purpose earlier
    while let Type::Paren(paren) = self_type {
        self_type = *paren.elem;
    }

    let vis = mem::replace(&mut function.vis, Visibility::Inherited);

    let mut declaration = function.sig.clone();

    // remove patterns from param names in the trait method declaration
    declaration.inputs.iter_mut().for_each(|arg| match arg {
        FnArg::Receiver(receiver) => {
            if receiver.reference.is_none() {
                receiver.mutability = None;
            }
        }
        FnArg::Typed(typed) => {
            typed.pat = parse_quote!(_);
        }
    });

    let trait_name = format_ident!("{}", function.sig.ident.to_string());

    let (impl_generics, ty_generics, _where_clause) = generics.split_for_impl();

    // filter out doc attributes
    let docs = function
        .attrs
        .iter()
        .filter(|attr| {
            attr.style == AttrStyle::Outer
                && matches!(&attr.meta, Meta::NameValue(meta) if meta.path == parse_quote!(doc))
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        #(#docs)*
        #vis trait #trait_name #impl_generics {
            #(#docs)*
            #[allow(async_fn_in_trait, unknown_lints, clippy::allow_attributes)]
            #declaration;
        }

        impl #impl_generics #trait_name #ty_generics for #self_type {
            #function
        }
    };

    Ok(expanded)
}

fn simplify_self_param(self_param: &mut Receiver) {
    self_param.colon_token = None;
    if let Type::Reference(ty) = *self_param.ty.clone() {
        if ty.elem == parse_quote!(Self) {
            self_param.reference = Some((ty.and_token, ty.lifetime));
            self_param.mutability = ty.mutability;
        }
    }
}

fn move_bounds_to_where_clause(
    params: &mut Punctuated<GenericParam, Comma>,
    where_clause: &mut Option<WhereClause>,
) {
    let where_predicates = params
        .iter_mut()
        .filter_map(|generic_param| match generic_param {
            GenericParam::Lifetime(lifetime_param) if !lifetime_param.bounds.is_empty() => {
                Some(WherePredicate::Lifetime(PredicateLifetime {
                    lifetime: lifetime_param.lifetime.clone(),
                    colon_token: Default::default(),
                    bounds: mem::take(&mut lifetime_param.bounds),
                }))
            }
            GenericParam::Type(type_param) if !type_param.bounds.is_empty() => {
                Some(WherePredicate::Type(PredicateType {
                    lifetimes: None,
                    bounded_ty: {
                        let ident = &type_param.ident;
                        parse_quote!(#ident)
                    },
                    colon_token: Default::default(),
                    bounds: mem::take(&mut type_param.bounds),
                }))
            }
            GenericParam::Lifetime(_) | GenericParam::Type(_) | GenericParam::Const(_) => None,
        });

    where_clause
        .get_or_insert_with(|| WhereClause {
            where_token: Default::default(),
            predicates: Punctuated::new(),
        })
        .predicates
        .extend(where_predicates);
}

/// Extract all generics from `generics` that are used in `ty`.
fn extract_impl_generics(ty: &Type, generics: &mut Generics) -> Generics {
    let fn_generic_params = mem::take(&mut generics.params);
    let mut extracted = Generics::default();
    let mut leftovers = Generics::default();

    for pair in fn_generic_params.into_pairs() {
        if type_uses_generic_param(ty, pair.value()) {
            extracted.params.extend([pair]);
        } else {
            leftovers.params.extend([pair]);
        }
    }

    // pop trailing commas
    extracted.params.pop_punct();
    leftovers.params.pop_punct();

    generics.params = leftovers.params;
    extracted
}

fn type_uses_generic_param(ty: &Type, generic_param: &GenericParam) -> bool {
    let mut visitor = FindGenericParam::new(generic_param);
    visitor.visit_type(ty);
    visitor.found
}

struct FindGenericParam<'gp> {
    generic_param: &'gp GenericParam,
    found: bool,
}

impl<'gp> FindGenericParam<'gp> {
    pub fn new(generic_param: &'gp GenericParam) -> Self {
        Self {
            generic_param,
            found: false,
        }
    }
}

impl<'ast, 'gp> Visit<'ast> for FindGenericParam<'gp> {
    // lifetime generics
    fn visit_lifetime(&mut self, node: &'ast Lifetime) {
        if let GenericParam::Lifetime(lifetime_param) = self.generic_param {
            self.found |= lifetime_param.lifetime.ident == node.ident;
        }
        visit::visit_lifetime(self, node);
    }

    // const generics
    fn visit_type_array(&mut self, node: &'ast TypeArray) {
        if let GenericParam::Const(const_param) = self.generic_param {
            self.found |= const_param.ident == node.len.to_token_stream().to_string();
        }
        visit::visit_type_array(self, node);
    }

    // type generics
    fn visit_type_path(&mut self, node: &'ast TypePath) {
        if let GenericParam::Type(type_param) = self.generic_param {
            self.found |= type_param.ident == node.to_token_stream().to_string();
        }
        visit::visit_type_path(self, node);
    }
}

struct ImplTraitsIntoGenerics<'g> {
    generics: &'g mut Generics,
    counter: usize,
}

impl<'g> ImplTraitsIntoGenerics<'g> {
    fn new(generics: &'g mut Generics) -> Self {
        Self {
            generics,
            counter: 0,
        }
    }
}

impl VisitMut for ImplTraitsIntoGenerics<'_> {
    fn visit_type_mut(&mut self, node: &mut Type) {
        visit_mut::visit_type_mut(self, node);

        if let Type::ImplTrait(impl_trait) = node.clone() {
            self.counter += 1;
            let ident = format_ident!("_T{}", self.counter);
            *node = parse_quote!(#ident);
            self.generics.params.push(GenericParam::Type(TypeParam {
                attrs: vec![],
                ident,
                colon_token: None,
                bounds: impl_trait.bounds,
                eq_token: None,
                default: None,
            }));
        };
    }
}
