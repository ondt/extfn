use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::mem;
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;
use syn::{
    Error, FnArg, GenericParam, Generics, ItemFn, Receiver, Result, Type, TypeParam,
    parse_macro_input, parse_quote, visit_mut,
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

    // take the type from `self`
    let mut self_type = *mem::replace(&mut self_param.ty, parse_quote!(Self));
    self_param.colon_token = None;

    let mut generics = mem::take(&mut function.sig.generics); // TODO: take only generics mentioned in `self`

    // convert `impl Trait` into `T: Trait`
    ImplTraitsIntoGenerics::new(&mut generics).visit_type_mut(&mut self_type);

    let mut declaration = function.sig.clone();

    // remove patterns from param names in the trait method declaration
    *declaration.inputs.first_mut().unwrap() = FnArg::Receiver(Receiver {
        attrs: vec![],
        reference: None,
        mutability: None,
        self_token: Default::default(),
        colon_token: None,
        ty: Box::new(parse_quote!(Self)),
    });
    declaration
        .inputs
        .iter_mut()
        .skip(1)
        .enumerate()
        .for_each(|(index, arg)| match arg {
            FnArg::Receiver(_) => unreachable!(),
            FnArg::Typed(param) => {
                let ident = format_ident!("dummy{index}");
                param.pat = Box::new(parse_quote!(#ident))
            }
        });

    let trait_name = format_ident!("{}", function.sig.ident.to_string());

    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();

    // TODO: seal
    let expanded = quote! {
        trait #trait_name {
            #declaration;
        }

        impl #impl_generics #trait_name for #self_type  #where_clause {
            #function
        }
    };

    Ok(expanded)
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
