#![allow(unused_mut)]
extern crate proc_macro;

#[macro_use]
mod quote;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn gen(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut in_fn = false;
    let mut output = TokenStream::new();
    let mut fn_name_changed = false;
    let mut body_brace_encountered = false;

    let mut is_mut = attr.to_string().contains("mut");
    let mut is_bin = attr.to_string().contains("bin");
    let mut is_libc = attr.to_string().contains("libc");
    let mut is_ret = !attr.to_string().contains("no_ret");
    let mut is_iter = !attr.to_string().contains("no_iter");
    let mut is_unsafe = !attr.to_string().contains("safe");
    let mut is_prelude = !attr.to_string().contains("no_prelude");

    let imports = quote! {
        ?(is_bin => #[cfg_attr(feature = "bin", macro_use)])
        ?(!is_bin => #[macro_use])
        extern crate macros;

        ?(is_bin && is_prelude => #[cfg(feature = "bin")])
        ?(is_bin && is_prelude => extern crate prelude;)

        ?(is_prelude => use prelude::*;)
        ?(is_libc => extern crate libc;)
    };

    let cfg_attr = quote! {
        ?(is_bin => #[cfg_attr(feature = "bin", start)])
        ?(!is_bin => #[start])
    };

    output.extend(imports);
    output.extend(cfg_attr);

    for token in item {
        match token {
            TokenTree::Ident(ref ident) if ident.to_string() == "fn" => {
                in_fn = true;
                output.extend(quote!(pub fn));
            }
            TokenTree::Ident(ref _ident) if in_fn && !fn_name_changed => {
                fn_name_changed = true;
                output.extend(quote!(_start));
            }
            TokenTree::Group(ref group) if in_fn && group.delimiter() == Delimiter::Parenthesis => {
                output.extend(quote!((argc: isize, argv: *const *const u8)));
            }
            TokenTree::Punct(ref punct) if punct.as_char() == '!' => {
                output.extend(quote!(isize));
            }
            TokenTree::Group(ref group)
                if in_fn && group.delimiter() == Delimiter::Brace && !body_brace_encountered =>
            {
                body_brace_encountered = true;
                let mut body = TokenStream::new();

                if !is_iter && is_bin {
                    is_bin = false;
                }

                body.extend(quote! {
                    let (program, c_args) = prelude::parse_args(argc, argv);
                    let ?(is_mut => mut) args = c_args?(is_bin => .into_iter());

                    ?(is_unsafe => unsafe { #(group.stream()); })
                    ?(!is_unsafe => #(group.stream());)

                    ?(is_ret => return 0;)
                });

                export!(output, { body });
            }
            _ => output.extend(std::iter::once(token)),
        }
    }

    return output;
}
