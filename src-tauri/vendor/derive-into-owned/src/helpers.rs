use crate::field_kind::FieldKind;

pub fn has_lifetime_arguments(segments: &[syn::PathSegment]) -> bool {
    if let Some(&syn::PathArguments::AngleBracketed(ref generics)) =
        segments.last().map(|x| &x.arguments)
    {
        generics
            .args
            .iter()
            .any(|f| matches!(f, syn::GenericArgument::Lifetime(_)))
    } else {
        false
    }
}

pub fn number_of_type_arguments(segments: &[syn::PathSegment]) -> usize {
    if let Some(&syn::PathArguments::AngleBracketed(ref generics)) =
        segments.last().map(|x| &x.arguments)
    {
        generics
            .args
            .iter()
            .filter(|f| matches!(f, syn::GenericArgument::Type(_)))
            .count()
    } else {
        0
    }
}

pub fn has_binding_arguments(segments: &[syn::PathSegment]) -> bool {
    if let Some(&syn::PathArguments::AngleBracketed(ref generics)) =
        segments.last().map(|x| &x.arguments)
    {
        generics
            .args
            .iter()
            .any(|f| matches!(f, syn::GenericArgument::Binding(_)))
    } else {
        false
    }
}

fn type_hopefully_is(segments: &[syn::PathSegment], expected: &str) -> bool {
    let expected = expected
        .split("::")
        .map(|x| quote::format_ident!("{}", x))
        .collect::<Vec<_>>();
    if segments.len() > expected.len() {
        return false;
    }

    let expected = expected.iter().collect::<Vec<_>>();
    let segments = segments.iter().map(|x| &x.ident).collect::<Vec<_>>();

    for len in 0..expected.len() {
        if segments[..] == expected[expected.len() - len - 1..] {
            return true;
        }
    }

    false
}

pub fn is_cow(segments: &[syn::PathSegment]) -> bool {
    type_hopefully_is(segments, "std::borrow::Cow")
}

pub fn is_cow_alike(segments: &[syn::PathSegment]) -> bool {
    if let Some(&syn::PathArguments::AngleBracketed(ref _data)) =
        segments.last().map(|x| &x.arguments)
    {
        has_lifetime_arguments(segments)
    } else {
        false
    }
}

pub fn collect_segments(path: &syn::Path) -> Vec<syn::PathSegment> {
    path.segments.iter().cloned().collect::<Vec<_>>()
}

pub fn is_opt_cow(mut segments: Vec<syn::PathSegment>) -> Option<FieldKind> {
    let mut levels = 0;
    loop {
        if type_hopefully_is(&segments, "std::option::Option") {
            if let syn::PathSegment {
                arguments: syn::PathArguments::AngleBracketed(ref data),
                ..
            } = *segments.last().expect("last segment")
            {
                if has_lifetime_arguments(&segments) || has_binding_arguments(&segments) {
                    // Option<&'a ?> cannot be moved but let the compiler complain
                    // don't know about data bindings
                    break;
                }

                if number_of_type_arguments(&segments) != 1 {
                    // Option<A, B> probably means some other, movable option
                    break;
                }

                match *data.args.first().expect("first arg") {
                    syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                        // segments: ref next_segments,
                        ref path,
                        ..
                    })) => {
                        levels += 1;
                        segments = collect_segments(path);
                        continue;
                    }
                    _ => break,
                }
            }
        } else if is_cow(&segments) {
            return Some(FieldKind::OptField(levels, Box::new(FieldKind::PlainCow)));
        } else if is_cow_alike(&segments) {
            return Some(FieldKind::OptField(levels, Box::new(FieldKind::AssumedCow)));
        }

        break;
    }

    None
}

pub fn is_iter_field(mut segments: Vec<syn::PathSegment>) -> Option<FieldKind> {
    loop {
        // this should be easy to do for arrays as well..
        if type_hopefully_is(&segments, "std::vec::Vec") {
            if let syn::PathSegment {
                arguments: syn::PathArguments::AngleBracketed(ref data),
                ..
            } = *segments.last().expect("last segment")
            {
                if has_lifetime_arguments(&segments) || has_binding_arguments(&segments) {
                    break;
                }

                // if data.types.len() != 1 {
                if number_of_type_arguments(&segments) != 1 {
                    // TODO: this could be something like Vec<(u32, Bar<'a>)>?
                    break;
                }

                match *data.args.first().expect("first arg") {
                    syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                        // segments: ref next_segments,
                        ref path,
                        ..
                    })) => {
                        segments = collect_segments(path);
                        continue;
                    }
                    _ => break,
                }
            }
        } else if is_cow(&segments) {
            return Some(FieldKind::IterableField(Box::new(FieldKind::PlainCow)));
        } else if is_cow_alike(&segments) {
            return Some(FieldKind::IterableField(Box::new(FieldKind::AssumedCow)));
        }

        break;
    }

    None
}
