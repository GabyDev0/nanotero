use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Expr, ExprLit, Fields, Lit, Meta, parse_macro_input, spanned::Spanned};

fn unwrap_option_type(ty: &syn::Type) -> (bool, &syn::Type) {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        return (true, inner_type); // 👈 Si es Option<T>, devuelve T
                    }
                }
            }
        }
    }
    (false, ty) // 👈 If it is not an Option, return the original type
}

#[proc_macro_derive(TeroDeserialize, attributes(tero))]
pub fn proc_macro_deserialize(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    
    let fields = match &ast.data {
        Data::Struct(strct) => {
            match &strct.fields {
                Fields::Named(fields) => &fields.named,
                Fields::Unnamed(_) =>  {
                    return syn::Error::new(
                        ast.span(),
                        quote! { compile_error!("TeroDeserialize does not support tuple structures"); }
                    ).into_compile_error().into();
                },
                Fields::Unit => {
                    return syn::Error::new(
                        ast.span(),
                        quote! { compile_error!("TeroDeserialize does not support empty structures"); }
                    ).into_compile_error().into();
                }
            }
        },
        Data::Enum(_) => {
            return syn::Error::new(
                ast.span(),
                quote! { compile_error!("TeroDeserialize does not support enums"); }
            ).into_compile_error().into();
        },
        Data::Union(_) => {
            return syn::Error::new(
                ast.span(),
                quote! { compile_error!("TeroDeserialize does not support unions"); }
            ).into_compile_error().into();
        }
    };

    let mut field_idents = Vec::new();
    let mut var_idents = Vec::new();
    let mut field_names = Vec::new();
    let mut field_ty = Vec::new();
    let mut field_is_optional = Vec::new();
    let mut field_initializers = Vec::new();
    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let mut name = ident.to_string();
        let (is_opt,ty) = unwrap_option_type(&field.ty);
        field_idents.push(ident);
        var_idents.push(format_ident!("var_ident{}", ident));
        
        field_ty.push(ty);
        field_is_optional.push(is_opt);
        let var_ident = var_idents.last().unwrap();
        if is_opt {
            // Si es opcional, pasamos directamente el Option<T> temporal
            field_initializers.push(quote! { #var_ident });
        } else {
            // Si no es opcional, lo desenvolvemos de forma insegura pero veloz
            field_initializers.push(quote! { unsafe { #var_ident.unwrap_unchecked() } });
        }
        for attr in &field.attrs {
            if attr.path().is_ident("tero") {
                if let Meta::List(meta_list) = &attr.meta {
                    if let Ok(Meta::NameValue(nv)) = meta_list.parse_args::<Meta>() {
                        if nv.path.is_ident("name") {
                            if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = nv.value {
                                name = lit_str.value(); // 💡 We overwrite it with the personalised name
                            }
                        }
                    }
                }
            }
        }
        field_names.push(name);
    } 

    let code = quote! {
        impl nanotero::Deserialize for #name {
            fn deserialize_tero(__input_value: nanotero::eval::FieldValue, __lexer: &mut nanotero::eval::Lexer) -> Result<Self, nanotero::eval::EvalError> {
                #(
                    let mut #var_idents: Option<#field_ty> = None;
                )*

                if let nanotero::eval::FieldValue::Block(mut __scope_ast) = __input_value {
                    while let Some(__opt) = __scope_ast.next(__lexer) {
                        match __opt {
                            Ok(__field__c) => {
                                match unsafe { __field__c.span().as_str_unchecked(__lexer) } {
                                    #(
                                        #field_names => {
                                            if #var_idents.is_some() {
                                                return Err(nanotero::eval::EvalError::DuplicateKey(__field__c.span()));
                                            }
                                            match __field__c.val() {
                                                nanotero::eval::FieldValue::Literal(nanotero::eval::Token::Nil) if #field_is_optional => {},
                                                val =>{ #var_idents = Some(nanotero::Deserialize::deserialize_tero(val, __lexer)?) }
                                            }
                                            
                                        },
                                    )*
                                    _ => {} // Ignore unknown fields 
                                }
                                
                            }
                            Err(err) => return Err(err)
                        }
                    }
                    #(
                        if #var_idents.is_none() & !#field_is_optional {
                            return Err(nanotero::eval::EvalError::MissingField(Box::from(#field_names)));
                        }
                    )*

                    Ok(#name {
                        #(
                            #field_idents: #field_initializers,
                        )*
                    })
                }else{
                    return Err(nanotero::eval::EvalError::UnexpectedToken(__input_value.as_token()));
                }
            }
        }
    };

    code.into()
}