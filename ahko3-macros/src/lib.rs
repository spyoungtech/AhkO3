use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn, ReturnType, Type, FnArg, Pat};

fn get_ahk_type(ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Path(type_path) if type_path.path.segments.last().unwrap().ident == "String" => {
            quote!(*const u16)
        }
        Type::Path(type_path) if type_path.path.segments.last().unwrap().ident == "i64" => {
            quote!(i64)
        }
        // Add more type mappings as needed
        _ => panic!("Unsupported type in AHK function"),
    }
}

fn get_ahk_return_type(ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Path(type_path) if type_path.path.segments.last().unwrap().ident == "String" => {
            quote!(*const u16)
        }
        Type::Path(type_path) if type_path.path.segments.last().unwrap().ident == "i64" => {
            quote!(std::ffi::c_longlong)
        }
        // Add more type mappings as needed
        _ => panic!("Unsupported return type in AHK function"),
    }
}

fn get_conversion_code(param_name: &syn::Ident, ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Path(type_path) if type_path.path.segments.last().unwrap().ident == "String" => {
            quote! {
                ahk_str_to_string(#param_name).unwrap()
            }
        }
        Type::Path(type_path) if type_path.path.segments.last().unwrap().ident == "i64" => {
            quote! { #param_name }
        }
        // Add more conversion codes as needed
        _ => panic!("Unsupported type conversion in AHK function"),
    }
}

fn get_return_conversion_code(ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Path(type_path) if type_path.path.segments.last().unwrap().ident == "String" => {
            quote! {
                string_to_ahk_ptr(result)
            }
        }
        Type::Path(type_path) if type_path.path.segments.last().unwrap().ident == "i64" => {
            quote! { result }
        }

        _ => quote! { result.into() }
    }
}


#[proc_macro_attribute]
pub fn ahkfunction(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let orig_fn_name = &input_fn.sig.ident;
    let gen_fn_name = format_ident!("gen_{}", orig_fn_name);

    // Extract parameter names and types
    let (param_names, param_types): (Vec<_>, Vec<_>) = input_fn.sig.inputs.iter().map(|arg| {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                (&pat_ident.ident, &*pat_type.ty)
            } else {
                panic!("Unsupported parameter pattern")
            }
        } else {
            panic!("Unsupported parameter type")
        }
    }).unzip();

    // Generate AHK parameter names
    let ahk_param_names = param_names.iter().map(|name| format_ident!("{}_ahk", name));

    // Generate AHK parameter types
    let ahk_param_types = param_types.iter().map(|ty| get_ahk_type(ty));

    // Generate conversion code for each parameter
    let param_conversions = param_names.iter().zip(param_types.iter()).map(|(name, ty)| {
        let ahk_name = format_ident!("{}_ahk", name);
        let conversion = get_conversion_code(&ahk_name, ty);
        quote! {
            let #name = #conversion;
        }
    });


    let (return_type, return_conversion) = match &input_fn.sig.output {
        ReturnType::Default => (quote!(()), quote!(())),
        ReturnType::Type(_, ty) => {
            let ahk_return_type = get_ahk_return_type(ty);
            let conversion = get_return_conversion_code(ty);
            (ahk_return_type, conversion)
        }
    };




    let expanded = quote! {
        #input_fn

        #[unsafe(no_mangle)]
        pub extern "C" fn #gen_fn_name(#(#ahk_param_names: #ahk_param_types),*) -> #return_type {
            #(#param_conversions)*
            let result = #orig_fn_name(#(#param_names),*);
            #return_conversion
        }
    };

    expanded.into()
}
