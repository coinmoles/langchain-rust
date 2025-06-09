use syn::{GenericArgument, PathArguments, Type};

pub fn extract_option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(p) = ty else {
        return None;
    };
    let last = p.path.segments.last()?;
    if last.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };
    let Some(GenericArgument::Type(inner)) = args.args.first() else {
        return None;
    };

    Some(inner)
}

pub fn is_str_type(ty: &Type) -> bool {
    let Type::Reference(r) = ty else {
        return false;
    };

    let Type::Path(p) = &*r.elem else {
        return false;
    };

    p.path.is_ident("str")
}

pub fn is_string_type(ty: &Type) -> bool {
    let Type::Path(p) = ty else {
        return false;
    };

    p.path.is_ident("String")
}

pub fn is_cow_str_type(ty: &Type) -> bool {
    let Type::Path(p) = ty else {
        return false;
    };

    let Some(seg) = p.path.segments.last() else {
        return false;
    };

    if seg.ident != "Cow" {
        return false;
    }

    let PathArguments::AngleBracketed(args) = &seg.arguments else {
        return false;
    };

    args.args.iter().any(|arg| match arg {
        GenericArgument::Type(Type::Path(p)) => p.path.is_ident("str"),
        _ => false,
    })
}

pub fn is_vec_message_type(ty: &Type) -> bool {
    let Type::Path(p) = ty else {
        return false;
    };

    let Some(seg) = p.path.segments.last() else {
        return false;
    };

    if seg.ident != "Vec" {
        return false;
    }

    let PathArguments::AngleBracketed(args) = &seg.arguments else {
        return false;
    };

    let Some(GenericArgument::Type(Type::Path(p))) = args.args.first() else {
        return false;
    };

    p.path
        .segments
        .last()
        .is_some_and(|seg| seg.ident == "Message")
}

pub fn is_message_slice_type(ty: &Type) -> bool {
    let Type::Slice(s) = ty else {
        return false;
    };

    let Type::Path(p) = &*s.elem else {
        return false;
    };

    p.path
        .segments
        .last()
        .is_some_and(|seg| seg.ident == "Message")
}
