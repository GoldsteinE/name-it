// lint me harder
#![forbid(non_ascii_idents)]
#![deny(
    future_incompatible,
    keyword_idents,
    elided_lifetimes_in_paths,
    meta_variable_misuse,
    noop_method_call,
    pointer_structural_match,
    unused_lifetimes,
    unused_qualifications,
    clippy::wildcard_dependencies,
    clippy::debug_assert_with_mut_call,
    clippy::empty_line_after_outer_attr,
    clippy::panic,
    clippy::unwrap_used,
    clippy::redundant_field_names,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::unneeded_field_pattern,
    clippy::useless_let_if_seq
)]
#![warn(clippy::pedantic)]

use std::iter;

use proc_macro::{Span, TokenStream};

use proc_macro_error::{abort, proc_macro_error, OptionExt, ResultExt as _};
use quote::{format_ident, quote};

struct SignatureVisitor;

impl syn::visit_mut::VisitMut for SignatureVisitor {
    fn visit_lifetime_mut(&mut self, i: &mut syn::Lifetime) {
        let replace = match &*i.ident.to_string() {
            "static" | "fut" => false,
            "_" => true,
            _ => {
                abort!(
                    i,
                    "custom lifetimes in #[name_it] args are not supported yet"
                );
            }
        };

        if replace {
            i.ident = syn::Ident::new("fut", Span::call_site().into());
        }

        syn::visit_mut::visit_lifetime_mut(self, i);
    }

    fn visit_type_reference_mut(&mut self, i: &mut syn::TypeReference) {
        if i.lifetime.is_none() {
            i.lifetime = Some(syn::Lifetime::new("'_", Span::call_site().into()));
        }

        syn::visit_mut::visit_type_reference_mut(self, i);
    }

    fn visit_type_trait_object_mut(&mut self, i: &mut syn::TypeTraitObject) {
        let mut found = false;
        for bound in &i.bounds {
            if matches!(bound, syn::TypeParamBound::Lifetime(_)) {
                found = true;
                break;
            }
        }

        if !found {
            i.bounds
                .push(syn::TypeParamBound::Lifetime(syn::Lifetime::new(
                    "'_",
                    Span::call_site().into(),
                )));
        }

        syn::visit_mut::visit_type_trait_object_mut(self, i);
    }
}

fn bump_visibility(vis: syn::Visibility) -> syn::Visibility {
    let super_ = syn::Ident::new("super", Span::call_site().into());
    match vis {
        syn::Visibility::Public(_) | syn::Visibility::Crate(_) => vis,
        syn::Visibility::Restricted(mut vis) => {
            let first = vis
                .path
                .segments
                .first_mut()
                .expect_or_abort("empty path in visibility declration");
            if first.ident == "self" {
                first.ident = super_;
                return syn::Visibility::Restricted(vis);
            }

            vis.path.segments.insert(
                0,
                syn::PathSegment {
                    ident: super_,
                    arguments: syn::PathArguments::None,
                },
            );
            syn::Visibility::Restricted(vis)
        }
        syn::Visibility::Inherited => syn::Visibility::Restricted(syn::VisRestricted {
            pub_token: syn::token::Pub::default(),
            paren_token: syn::token::Paren::default(),
            in_token: None,
            path: Box::new(syn::Path {
                leading_colon: None,
                segments: std::iter::once(syn::PathSegment {
                    ident: super_,
                    arguments: syn::PathArguments::None,
                })
                .collect(),
            }),
        }),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn name_it(attr: TokenStream, func: TokenStream) -> TokenStream {
    let mut func: syn::ItemFn =
        syn::parse(func).expect_or_abort("#[name_it] accepts only a function");
    let type_name: syn::Ident =
        syn::parse(attr).expect_or_abort("#[name_it] sole argument must be an ident");

    if func.sig.asyncness.is_none() {
        abort!(func, "#[name_it] only works on async functions");
    }

    if !func.sig.generics.params.is_empty() {
        abort!(
            func.sig.generics,
            "generics are not supported by #[name_it] yet"
        );
    }

    let func_name = func.sig.ident.clone();
    let mut func_return_type = func.sig.output.clone();
    if func_return_type == syn::ReturnType::Default {
        func_return_type = syn::ReturnType::Type(
            <syn::Token![->]>::default(),
            Box::new(syn::Type::Tuple(syn::TypeTuple {
                paren_token: syn::token::Paren::default(),
                elems: syn::punctuated::Punctuated::new(),
            })),
        );
    }

    let mut wrapped_func = func.clone();
    wrapped_func.sig.asyncness = None;
    let fut_lifetime = syn::Lifetime::new("'fut", Span::call_site().into());
    let mut generics = syn::Generics::default();
    generics
        .params
        .push(syn::GenericParam::Lifetime(syn::LifetimeDef::new(
            fut_lifetime.clone(),
        )));
    wrapped_func.sig.generics = generics;
    wrapped_func.sig.output = syn::ReturnType::Type(
        <syn::Token![->]>::default(),
        Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: iter::once(syn::PathSegment {
                    ident: type_name.clone(),
                    arguments: syn::PathArguments::AngleBracketed(
                        syn::AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: <syn::Token![<]>::default(),
                            args: [syn::GenericArgument::Lifetime(fut_lifetime)]
                                .into_iter()
                                .collect(),
                            gt_token: <syn::Token![>]>::default(),
                        },
                    ),
                })
                .collect(),
            },
        })),
    );
    let arg_idents = func.sig.inputs.iter().map(|arg| match arg {
        syn::FnArg::Receiver(_) => {
            abort!(arg, "methods are not supported by #[name_it]");
        }
        syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
            syn::Pat::Ident(ident)
                if ident.by_ref.is_none() && ident.subpat.is_none() && ident.attrs.is_empty() =>
            {
                if matches!(&*pat_type.ty, syn::Type::ImplTrait(_)) {
                    abort!(pat_type.ty, "generics are not supported by #[name_it] yet");
                }

                ident.ident.clone()
            }
            _ => abort!(
                arg,
                "only simple `ident` patterns in function args are supported by #[name_it] for now"
            ),
        },
    });
    syn::visit_mut::visit_signature_mut(&mut SignatureVisitor, &mut wrapped_func.sig);

    let vis = func.vis.clone();
    func.vis = bump_visibility(func.vis);
    let new_vis = func.vis.clone();

    let module_name = format_ident!("_{}_impl", func_name);
    func.sig.ident = func_name.clone();
    wrapped_func.block = Box::new(
        syn::parse(
            quote! {{
                let fut = #module_name::#func_name(#(#arg_idents),*);
                // SAFETY:
                // 1. type and alignment are the same, so transmuting to array of `MaybeUninit<u8>`
                //    is always ok
                // 2. we pass these bytes into `::new()` of the corresponding type
                unsafe {
                    let bytes = ::name_it::transmute_generic(fut);
                    ::name_it::Named::new(#module_name::#type_name::new(bytes))
                }
            }}
            .into(),
        )
        .expect("failed to parse function block from procmacro, this is a bug"),
    );

    let underscores = func.sig.inputs.iter().map(|_| syn::TypeInfer {
        underscore_token: <syn::Token![_]>::default(),
    });

    quote! {
        mod #module_name {
            use super::*;

            #[forbid(elided_lifetimes_in_paths)]
            #func

            ::name_it::_name_it_inner!(#new_vis type #type_name = #func_name(#(#underscores),*) #func_return_type);
        }

        #vis type #type_name<'fut> = ::name_it::Named<#module_name::#type_name<'fut>>;

        #[allow(unused_mut)]
        #wrapped_func
    }
    .into()
}
