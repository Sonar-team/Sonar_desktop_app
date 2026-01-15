use quote::{format_ident, quote};

use crate::helpers::{collect_segments, is_cow, is_cow_alike, is_iter_field, is_opt_cow};

#[derive(Debug)]
pub enum FieldKind {
    PlainCow,
    AssumedCow,
    /// Option fields with either PlainCow or AssumedCow
    OptField(usize, Box<FieldKind>),
    IterableField(Box<FieldKind>),
    JustMoved,
}
impl FieldKind {
    pub fn resolve(ty: &syn::Type) -> Self {
        if let syn::Type::Path(syn::TypePath { ref path, .. }) = ty {
            if is_cow(&collect_segments(path)) {
                FieldKind::PlainCow
            } else if is_cow_alike(&collect_segments(path)) {
                FieldKind::AssumedCow
            } else if let Some(kind) = is_opt_cow(collect_segments(path)) {
                kind
            } else if let Some(kind) = is_iter_field(collect_segments(path)) {
                kind
            } else {
                FieldKind::JustMoved
            }
        } else {
            FieldKind::JustMoved
        }
    }

    pub fn move_or_clone_field(&self, var: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        use self::FieldKind::*;

        match *self {
            PlainCow => quote! { ::std::borrow::Cow::Owned(#var.into_owned()) },
            AssumedCow => quote! { #var.into_owned() },
            OptField(levels, ref inner) => {
                let next = format_ident!("val");
                let next = quote! { #next };

                let mut tokens = inner.move_or_clone_field(&next);

                for _ in 0..(levels - 1) {
                    tokens = quote! { #next.map(|#next| #tokens) };
                }

                quote! { #var.map(|#next| #tokens) }
            }
            IterableField(ref inner) => {
                let next = format_ident!("x");
                let next = quote! { #next };

                let tokens = inner.move_or_clone_field(&next);

                quote! { #var.into_iter().map(|x| #tokens).collect() }
            }
            JustMoved => quote! { #var },
        }
    }

    pub fn borrow_or_clone(&self, var: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        use self::FieldKind::*;

        match *self {
            PlainCow => quote! { ::std::borrow::Cow::Borrowed(#var.as_ref()) },
            AssumedCow => quote! { #var.borrowed() },
            OptField(levels, ref inner) => {
                let next = format_ident!("val");
                let next = quote! { #next };

                let mut tokens = inner.borrow_or_clone(&next);

                for _ in 0..(levels - 1) {
                    tokens = quote! { #next.as_ref().map(|#next| #tokens) };
                }

                quote! { #var.as_ref().map(|#next| #tokens) }
            }
            IterableField(ref inner) => {
                let next = format_ident!("x");
                let next = quote! { #next };

                let tokens = inner.borrow_or_clone(&next);

                quote! { #var.iter().map(|x| #tokens).collect() }
            }
            JustMoved => quote! { #var.clone() },
        }
    }
}
