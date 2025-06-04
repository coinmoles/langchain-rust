use syn::{GenericArgument, PathArguments, Type};

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
