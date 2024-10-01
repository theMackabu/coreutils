extern crate proc_macro;
use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn gen(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut in_fn = false;
    let mut output = TokenStream::new();
    let mut fn_name_changed = false;
    let mut body_brace_encountered = false;

    let cfg_attr = match attr.to_string().contains("bin") {
        false => "start",
        true => "cfg_attr(feature = \"bin\", start)",
    };

    output.extend(vec![
        TokenTree::Punct(Punct::new('#', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Bracket, cfg_attr.parse().unwrap())),
    ]);

    for token in item {
        match token {
            TokenTree::Ident(ref ident) if ident.to_string() == "fn" => {
                in_fn = true;
                output.extend(vec![TokenTree::Ident(Ident::new("pub", ident.span())), token]);
            }
            TokenTree::Ident(ref ident) if in_fn && !fn_name_changed => {
                fn_name_changed = true;
                output.extend(vec![TokenTree::Ident(Ident::new("_start", ident.span()))]);
            }
            TokenTree::Group(ref group) if in_fn && group.delimiter() == Delimiter::Parenthesis => {
                output.extend(vec![TokenTree::Group(Group::new(Delimiter::Parenthesis, "argc: isize, argv: *const *const u8".parse().unwrap()))]);
            }
            TokenTree::Punct(ref punct) if punct.as_char() == '!' => {
                output.extend(vec![TokenTree::Ident(Ident::new("isize", punct.span()))]);
            }
            TokenTree::Group(ref group) if in_fn && group.delimiter() == Delimiter::Brace && !body_brace_encountered => {
                body_brace_encountered = true;
                let mut new_body = TokenStream::new();
                new_body.extend(vec![
                    TokenTree::Ident(Ident::new("let", group.span())),
                    TokenTree::Ident(Ident::new("args", group.span())),
                    TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("prelude", group.span())),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("parse_args", group.span())),
                    TokenTree::Group(Group::new(Delimiter::Parenthesis, "argc, argv".parse().unwrap())),
                    TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("into_iter", group.span())),
                    TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
                    TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                ]);
                new_body.extend(group.stream());
                output.extend(vec![TokenTree::Group(Group::new(Delimiter::Brace, new_body))]);
            }
            _ => output.extend(vec![token]),
        }
    }

    return output;
}
