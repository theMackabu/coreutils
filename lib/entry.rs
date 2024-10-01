extern crate proc_macro;
use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn gen(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();

    let add_start_attr = attr.to_string().contains("start");

    if add_start_attr {
        output.extend(vec![
            TokenTree::Punct(Punct::new('#', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Bracket, "cfg_attr(feature = \"start\", start)".parse().unwrap())),
        ]);
    }

    let mut in_fn = false;
    let mut fn_name_changed = false;

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
            _ => output.extend(vec![token]),
        }
    }

    return output;
}
