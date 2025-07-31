use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, Expr, ExprLit, ExprPath, ItemFn, Lit, Meta, MetaNameValue, Token,
};

#[proc_macro_attribute]
pub fn on_ok(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as Expr);
    let input_fn = parse_macro_input!(item as ItemFn);

    let on_ok_fn: Option<ExprPath> = if let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str), ..
    }) = args
    {
        Some(lit_str.parse().unwrap())
    } else {
        None
    };

    let on_ok = on_ok_fn.expect("Missing callback");

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;

    let gen = quote! {
        #vis #sig {
            let result = (|| #block)();
            match &result {
                Ok(_) => #on_ok(),
                Err(_) => (),
            }
            result
        }
    };

    gen.into()
}

#[proc_macro_attribute]
pub fn on_result(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr with Punctuated::<Meta, Token![,]>::parse_terminated);
    let input_fn = parse_macro_input!(item as ItemFn);

    // Default callback names
    let mut on_ok_fn: Option<ExprPath> = None;
    let mut on_err_fn: Option<ExprPath> = None;

    for arg in args {
        if let Meta::NameValue(MetaNameValue {
            path,
            value: Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str), ..
            }),
            ..
        }) = arg
        {
            let ident = path.get_ident().map(|i| i.to_string());
            match ident.as_deref() {
                Some("on_ok") => on_ok_fn = Some(lit_str.parse().unwrap()),
                Some("on_err") => on_err_fn = Some(lit_str.parse().unwrap()),
                _ => {}
            }
        }
    }

    let on_ok_call = on_ok_fn.map(|f| quote!(#f()));
    let on_err_call = on_err_fn.map(|f| quote!(#f()));

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;

    let gen = quote! {
        #vis #sig {
            let result = (|| #block)();
            match &result {
                Ok(_) => { #on_ok_call }
                Err(_) => { #on_err_call }
            }
            result
        }
    };

    gen.into()
}

#[proc_macro_attribute]
pub fn retry(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr with Punctuated::<Meta, Token![,]>::parse_terminated);
    let input_fn = parse_macro_input!(item as ItemFn);

    let mut times = 0;
    let mut delay_ms = 0;

    for arg in args {
        if let Meta::NameValue(MetaNameValue {
            path,
            value: Expr::Lit(ExprLit {
                lit: Lit::Int(lit_int), ..
            }),
            ..
        }) = arg
        {
            let ident = path.get_ident().map(|i| i.to_string());
            match ident.as_deref() {
                Some("times") => times = lit_int.base10_parse::<usize>().unwrap(),
                Some("delay_ms") => delay_ms = lit_int.base10_parse::<u64>().unwrap(),
                _ => {}
            }
        }
    }

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;

    let delay_call = if delay_ms > 0 {
        quote! {
            std::thread::sleep(std::time::Duration::from_millis(#delay_ms));
        }
    } else {
        quote! {}
    };

    let gen = quote! {
        #vis #sig {
            let mut result;
            for i in 0..#times {
                result = (|| #block)();
                if result.is_ok() {
                    return result;
                }
                if i < #times - 1 {
                    #delay_call
                }
            }
            (|| #block)()
        }
    };

    gen.into()
}

#[proc_macro_attribute]
pub fn timeout(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr with Punctuated::<Meta, Token![,]>::parse_terminated);
    let input_fn = parse_macro_input!(item as ItemFn);

    let mut duration_ms = 0;

    for arg in args {
        if let Meta::NameValue(MetaNameValue {
            path,
            value: Expr::Lit(ExprLit {
                lit: Lit::Int(lit_int), ..
            }),
            ..
        }) = arg
        {
            let ident = path.get_ident().map(|i| i.to_string());
            match ident.as_deref() {
                Some("duration_ms") => duration_ms = lit_int.base10_parse::<u64>().unwrap(),
                _ => {}
            }
        }
    }

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;

    let gen = quote! {
        #vis #sig {
            let (sender, receiver) = std::sync::mpsc::channel();
            let handle = std::thread::spawn(move || {
                let result = (|| #block)();
                sender.send(result).unwrap();
            });

            match receiver.recv_timeout(std::time::Duration::from_millis(#duration_ms)) {
                Ok(result) => result,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    Err(format!("Function timed out after {}ms", #duration_ms))
                },
                Err(e) => Err(format!("Channel error: {}", e)),
            }
        }
    };

    gen.into()
}

#[proc_macro_attribute]
pub fn hook(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr with Punctuated::<Meta, Token![,]>::parse_terminated);
    let input_fn = parse_macro_input!(item as ItemFn);

    let mut on_pre_fn: Option<ExprPath> = None;
    let mut on_post_fn: Option<ExprPath> = None;

    for arg in args {
        if let Meta::NameValue(MetaNameValue {
            path,
            value: Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str), ..
            }),
            ..
        }) = arg
        {
            let ident = path.get_ident().map(|i| i.to_string());
            match ident.as_deref() {
                Some("on_pre") => on_pre_fn = Some(lit_str.parse().unwrap()),
                Some("on_post") => on_post_fn = Some(lit_str.parse().unwrap()),
                _ => {}
            }
        }
    }

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;

    let pre_hook_code = if let Some(f) = on_pre_fn {
        quote! { #f(); }
    } else {
        quote! {}
    };

    let post_hook_code = if let Some(f) = on_post_fn {
        quote! { #f(); }
    } else {
        quote! {}
    };

    let gen = quote! {
        #vis #sig {
            #pre_hook_code
            let result = (|| #block)();
            #post_hook_code
            result
        }
    };

    gen.into()
}
