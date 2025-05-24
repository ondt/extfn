use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::mem;
use syn::spanned::Spanned;
use syn::{
    Error, FnArg, GenericParam, ItemFn, Result, Type, TypeParam, parse_macro_input, parse_quote,
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
    if let Type::ImplTrait(impl_trait) = self_type.clone() {
        let ident = format_ident!("__extfn_IMPL");
        self_type = parse_quote!(#ident);
        generics.params.push(GenericParam::Type(TypeParam {
            attrs: vec![],
            ident,
            colon_token: None,
            bounds: impl_trait.bounds.clone(),
            eq_token: None,
            default: None,
        }));
    };

    let declaration = function.sig.clone();
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
