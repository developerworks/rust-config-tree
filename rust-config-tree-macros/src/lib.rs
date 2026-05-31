use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Error, Fields, GenericArgument, LitStr, PathArguments, Type,
    parse_macro_input,
};

#[proc_macro_derive(ConfigOverrides, attributes(config_override))]
pub fn derive_config_overrides(input: TokenStream) -> TokenStream {
    match expand_config_overrides(parse_macro_input!(input as DeriveInput)) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(ConfigSchema, attributes(config_schema))]
pub fn derive_config_schema(input: TokenStream) -> TokenStream {
    match expand_config_schema(parse_macro_input!(input as DeriveInput)) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_config_overrides(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => {
                return Err(Error::new_spanned(
                    name,
                    "ConfigOverrides only supports structs with named fields",
                ));
            }
        },
        _ => {
            return Err(Error::new_spanned(
                name,
                "ConfigOverrides only supports structs",
            ));
        }
    };

    let mut inserts = Vec::new();
    for field in fields {
        let Some(path) = override_path(&field.attrs)? else {
            continue;
        };
        let ident = field.ident.ok_or_else(|| {
            Error::new_spanned(&field.ty, "config_override must be used on a named field")
        })?;

        if option_inner(&field.ty).is_some() {
            inserts.push(quote! {
                if let Some(value) = &self.#ident {
                    provider.insert(#path, value)?;
                }
            });
        } else {
            inserts.push(quote! {
                provider.insert(#path, &self.#ident)?;
            });
        }
    }

    Ok(quote! {
        impl #impl_generics ::rust_config_tree::cli::ConfigOverrides for #name #ty_generics #where_clause {
            fn config_overrides(
                &self,
            ) -> ::rust_config_tree::config::ConfigResult<::rust_config_tree::cli::ConfigOverrideProvider> {
                let mut provider = ::rust_config_tree::cli::ConfigOverrideProvider::new();
                #(#inserts)*
                Ok(provider)
            }
        }
    })
}

fn override_path(attrs: &[Attribute]) -> syn::Result<Option<LitStr>> {
    let mut path = None;

    for attr in attrs {
        if !attr.path().is_ident("config_override") {
            continue;
        }

        if path.is_some() {
            return Err(Error::new_spanned(
                attr,
                "config_override cannot be repeated on the same field",
            ));
        }

        let parsed_path = parse_override_path(attr)?;
        validate_path(&parsed_path)?;
        path = Some(parsed_path);
    }

    Ok(path)
}

fn parse_override_path(attr: &Attribute) -> syn::Result<LitStr> {
    if let Ok(path) = attr.parse_args::<LitStr>() {
        return Ok(path);
    }

    let mut path = None;
    attr.parse_nested_meta(|meta| {
        if !meta.path.is_ident("path") {
            return Err(meta.error("config_override only supports the path argument"));
        }
        let value = meta.value()?;
        let lit = value.parse::<LitStr>()?;
        path = Some(lit);
        Ok(())
    })?;

    path.ok_or_else(|| Error::new_spanned(attr, "config_override requires a path argument"))
}

fn validate_path(path: &LitStr) -> syn::Result<()> {
    let value = path.value();
    if value.is_empty() {
        return Err(Error::new_spanned(
            path,
            "config_override path must not be empty",
        ));
    }

    if value.split('.').any(str::is_empty) {
        return Err(Error::new_spanned(
            path,
            "config_override path must not contain empty segments",
        ));
    }

    Ok(())
}

fn option_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    let mut args = arguments.args.iter();
    let Some(GenericArgument::Type(inner)) = args.next() else {
        return None;
    };
    if args.next().is_some() {
        return None;
    }
    Some(inner)
}

// ---------------------------------------------------------------------------
// ConfigSchema derive
// ---------------------------------------------------------------------------

fn expand_config_schema(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => {
                return Err(Error::new_spanned(
                    &name,
                    "ConfigSchema only supports structs with named fields",
                ));
            }
        },
        _ => {
            return Err(Error::new_spanned(
                &name,
                "ConfigSchema only supports structs",
            ));
        }
    };

    // 1. Look for a field annotated with #[config_schema(include)].
    let mut include_field: Option<syn::Ident> = None;
    for field in &fields {
        if has_config_schema_include_attr(&field.attrs) {
            let ident = field.ident.clone().ok_or_else(|| {
                Error::new_spanned(&field.ty, "config_schema(include) must be on a named field")
            })?;
            include_field = Some(ident);
            break;
        }
    }

    // 2. Fall back to a field named `include` whose type is Vec<PathBuf>.
    if include_field.is_none() {
        for field in &fields {
            let ident = field.ident.as_ref().ok_or_else(|| {
                Error::new_spanned(&field.ty, "ConfigSchema requires named fields")
            })?;
            if ident == "include" && is_vec_path_buf(&field.ty) {
                include_field = Some(ident.clone());
                break;
            }
        }
    }

    let include_ident = include_field.ok_or_else(|| {
        Error::new_spanned(
            &name,
            "ConfigSchema requires a field for include paths. \
             Annotate one with #[config_schema(include)] or name it `include: Vec<PathBuf>`.",
        )
    })?;

    Ok(quote! {
        impl #impl_generics ::rust_config_tree::config::ConfigSchema for #name #ty_generics #where_clause {
            fn include_paths(
                layer: &<Self as ::confique::Config>::Layer,
            ) -> ::std::vec::Vec<::std::path::PathBuf> {
                layer.#include_ident.clone().unwrap_or_default()
            }
        }
    })
}

/// Checks whether a field carries `#[config_schema(include)]`.
fn has_config_schema_include_attr(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if !attr.path().is_ident("config_schema") {
            continue;
        }
        // Accept `#[config_schema(include)]`.
        if attr
            .parse_args::<syn::Ident>()
            .is_ok_and(|ident| ident == "include")
        {
            return true;
        }
    }
    false
}

/// Returns `true` when the type is `Vec<PathBuf>` (with any leading
/// `std::` / `::std::` qualifiers).
fn is_vec_path_buf(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };
    let segment = match type_path.path.segments.last() {
        Some(s) => s,
        None => return false,
    };
    if segment.ident != "Vec" {
        return false;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return false;
    };
    let Some(GenericArgument::Type(inner)) = args.args.first() else {
        return false;
    };
    is_path_buf(inner)
}

/// Returns `true` when the type resolves to `PathBuf` (possibly qualified).
fn is_path_buf(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };
    let segment = match type_path.path.segments.last() {
        Some(s) => s,
        None => return false,
    };
    segment.ident == "PathBuf"
}
