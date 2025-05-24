use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::mem;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::{
    Error, FnArg, GenericParam, Generics, ItemFn, Lifetime, Result, Type, TypeArray, TypeParam,
    TypePath, parse_macro_input, parse_quote, visit, visit_mut,
};

#[proc_macro_attribute]
pub fn extfn(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let function = parse_macro_input!(input as ItemFn);
    expand(function)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand(mut function: ItemFn) -> Result<proc_macro2::TokenStream> {
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

    // extract all generics used in `self_type`
    let mut generics = extract_impl_generics(&self_type, &mut function.sig.generics);

    // convert `impl Trait` into `T: Trait`
    ImplTraitsIntoGenerics::new(&mut generics).visit_type_mut(&mut self_type);

    // strip extra parenthesis that might've had a purpose earlier
    while let Type::Paren(paren) = self_type {
        self_type = *paren.elem;
    }

    let mut declaration = function.sig.clone();

    // remove patterns from param names in the trait method declaration
    declaration
        .inputs
        .iter_mut()
        .enumerate()
        .for_each(|(index, arg)| match arg {
            FnArg::Receiver(receiver) => {
                receiver.reference = None;
                receiver.mutability = None;
            }
            FnArg::Typed(typed) => {
                let ident = format_ident!("dummy{index}");
                typed.pat = parse_quote!(#ident)
            }
        });

    let trait_name = format_ident!("{}", function.sig.ident.to_string());

    let (impl_generics, ty_generics, _where_clause) = generics.split_for_impl();

    // TODO: seal
    let expanded = quote! {
        trait #trait_name #impl_generics {
            #[expect(clippy::needless_arbitrary_self_type)]
            #declaration;
        }

        impl #impl_generics #trait_name #ty_generics for #self_type {
            #[expect(clippy::needless_arbitrary_self_type)]
            #function
        }
    };

    Ok(expanded)
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

        visit_mut::visit_type_mut(self, node);
    }
}
