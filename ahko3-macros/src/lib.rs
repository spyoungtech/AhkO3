use proc_macro::TokenStream;
use std::process::id;
use proc_macro2::Ident;
use quote::{quote, format_ident, ToTokens};
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
        Type::Path(type_path) => {
            let ident = &type_path.path.segments.last().unwrap().ident;
            match ident.to_string().as_str() {

                // When AHK expects a string as a return parameter in DLLCall, we return a pointer to a WStr
                "String" => {
                    quote! { *const u16 }
                }

                "i64" => {quote! { std::ffi::c_longlong }}
                "i32" => {quote! { std::ffi::c_long }}
                "f32" => {quote! { std::ffi::c_float }}
                "f64" => {quote! { std::ffi::c_double }}
                "u32" => {quote! { std::ffi::c_ulong }}
                "u64" => {quote! { std::ffi::c_ulonglong }}
                "usize" => {quote! { std::ffi::c_longlong }}
                "isize" => {quote! { std::ffi::c_longlong }}

                _ => panic!("Unsupported return type in AHK function ({:?})", ident),
            }
        }
        // Add more conversion codes as needed
        _ => panic!("Unsupported return type in AHK function ({:?})", ty.to_token_stream()),
    }
}

fn get_parameter_conversion_code(param_name: &syn::Ident, ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Path(type_path) => {
            let ident = &type_path.path.segments.last().unwrap().ident;
            match ident.to_string().as_str() {

                // When AHK provides a string as a parameter, it comes in as a WStr, so it has to be converted first
                "String" => {
                    quote! { ahk_str_to_string(#param_name).unwrap() }
                }

                // these can be left as-is; AHK can call with the appropriate type, no conversions needed
                "i64" | "i32" | "u32" | "u64" | "f64" | "f32" => {quote! { #param_name }}
                _ => panic!("Unsupported type conversion in AHK function ({:?} {:?})", param_name, ident),
            }
        }
        _ => panic!("Unsupported type conversion in AHK function ({:?} {:?})", param_name, ty.to_token_stream()),
    }
}

fn get_return_conversion_code(ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Path(type_path) if type_path.path.segments.last().unwrap().ident == "String" => {
            quote! {
                string_to_ahk_ptr(result)
            }
        }

        _ => quote! { result.into() }
    }
}

struct FunctionArgument {
    arg_name: Ident,
    arg_type: Type,
}

impl FunctionArgument {
    fn get_ahk_type(&self) -> proc_macro2::TokenStream {
        get_ahk_type(&self.arg_type)
    }
}

struct FunctionMetadata {
    original_name: Ident,
    arguments: Vec<FunctionArgument>,
    return_type: ReturnType
}

impl FunctionMetadata {
    fn ahk_ffi_name(&self) -> Ident {
        let namespace = std::env::var("CARGO_PKG_NAME").unwrap_or(String::from("ahko3_unknown_namespace"));
        format_ident!("{}_{}", namespace, self.original_name)
    }

    fn ahk_ffi_parameters(&self) -> (Vec<Ident>, Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>){
        let mut param_names: Vec<Ident> = Vec::new();
        let mut param_types: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut conversions: Vec<proc_macro2::TokenStream> = Vec::new();
        for arg in self.arguments.iter() {
            let ahk_arg = format_ident!("{}_ahk", arg.arg_name);
            let arg_name = &arg.arg_name;
            param_names.push(ahk_arg.clone());
            param_types.push(arg.get_ahk_type());
            let conversion = get_parameter_conversion_code(&ahk_arg, &arg.arg_type);
            conversions.push(quote! {
                let #arg_name = #conversion;

            });
        }

        (param_names, param_types, conversions)
    }

    fn ahk_ffi_return_type(&self) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        match &self.return_type {
            ReturnType::Default => {
                (quote! {std::ffi::c_longlong}, quote! {0})
            }
            ReturnType::Type(_, ty) => {
                let rt = get_ahk_return_type(&*ty);
                let rc = get_return_conversion_code(&*ty);
                (rt, rc)
            }
        }
    }

    fn ahk_ffi_function(&self) -> proc_macro2::TokenStream {
        let gen_fn_name = self.ahk_ffi_name();
        let (ahk_param_names, ahk_param_types, conversions) = self.ahk_ffi_parameters();
        let (return_type, return_conversion) = self.ahk_ffi_return_type();
        let arg_names = self.arguments.iter().map(|arg| &arg.arg_name);
        let original_name = &self.original_name;
        quote! {
            #[unsafe(no_mangle)]
            pub extern "C" fn #gen_fn_name(#(#ahk_param_names: #ahk_param_types),*) -> #return_type {
                #(#conversions)*
                let result = #original_name(#(#arg_names),*);
                #return_conversion
            }
        }
    }


}

impl From<ItemFn> for FunctionMetadata {
    fn from(input_fn: ItemFn) -> Self {
        let original_name = input_fn.sig.ident;
        let args: Vec<FunctionArgument> = input_fn.sig.inputs.iter().map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    FunctionArgument{arg_name: pat_ident.ident.clone(), arg_type: *pat_type.ty.clone()}
                } else {
                    panic!("Unsupported parameter pattern")
                }
            } else {
                panic!("Unsupported parameter type")
            }
        }).collect();
        FunctionMetadata{original_name, arguments: args, return_type: input_fn.sig.output}
    }
}

#[proc_macro_attribute]
pub fn ahkfunction(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let metadata = FunctionMetadata::from(input_fn.clone());
    let expanded = metadata.ahk_ffi_function();
    let output = quote! {
        #input_fn

        #expanded
    };
    output.into()
}
