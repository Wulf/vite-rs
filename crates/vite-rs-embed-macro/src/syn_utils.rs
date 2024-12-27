use syn::{Data, Expr, ExprLit, Fields, Lit, Meta, MetaNameValue};

/// Find all pairs of the `name = "value"` attribute from the derive input
pub fn find_attribute_values(ast: &syn::DeriveInput, attr_name: &str) -> Vec<String> {
    ast.attrs
        .iter()
        .filter(|value| value.path().is_ident(attr_name))
        .filter_map(|attr| match &attr.meta {
            // `name = "value"`
            Meta::NameValue(MetaNameValue {
                value:
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(val), ..
                    }),
                ..
            }) => Some(val.value()),
            // `name = 123`
            Meta::NameValue(MetaNameValue {
                value:
                    Expr::Lit(ExprLit {
                        lit: Lit::Int(val), ..
                    }),
                ..
            }) => Some(val.base10_digits().to_string()),
            // other
            _ => None,
        })
        .collect()
}

/// Returns an Err if the DeriveInput is not a unit struct
///
/// # Example
///
/// ```ignore
/// #[derive(vite_rs::Embed)]
/// struct MyStruct;
/// ```
///
/// Instead of:
///
/// ```ignore
/// #[derive(vite_rs::Embed)]
/// struct MyStruct {
///    field: String,
/// }
/// ```
pub fn ensure_unit_struct(ast: &syn::DeriveInput) -> syn::Result<()> {
    match ast.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Unit => {}
            _ => {
                return Err(syn::Error::new_spanned(
                    ast,
                    "Embed can only be derived for unit structs",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                ast,
                "Embed can only be derived for unit structs",
            ))
        }
    };

    Ok(())
}
