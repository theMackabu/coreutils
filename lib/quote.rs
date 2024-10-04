macro_rules! export {
    ($output:ident, { $body:ident }) => {
        $output.extend(std::iter::once(TokenTree::Group(Group::new(Delimiter::Brace, $body))));
    };
}

macro_rules! quote {
    ($($tt:tt)*) => {{
        let mut output = TokenStream::new();
        quote_inner!(output, $($tt)*);
        output
    }};
}

macro_rules! quote_inner {
    ($output:ident, # [ $($inner:tt)* ] $($rest:tt)*) => {{
        $output.extend(std::iter::once(TokenTree::Punct(Punct::new('#', Spacing::Alone))));
        let mut inner = TokenStream::new();
        quote_inner!(inner, $($inner)*);
        $output.extend(std::iter::once(TokenTree::Group(Group::new(Delimiter::Bracket, inner))));
        quote_inner!($output, $($rest)*);
    }};
    ($output:ident, # ( $($expr:tt)+ ) $($rest:tt)*) => {{
        let expanded: TokenStream = $($expr)+;
        $output.extend(expanded);
        quote_inner!($output, $($rest)*);
    }};
    ($output:ident, ? ($cond:expr => $($token:tt)+) $($rest:tt)*) => {
        if $cond {
            quote_inner!($output, $($token)+ $($rest)*);
        } else {
            quote_inner!($output, $($rest)*);
        }
    };
    ($output:ident, unsafe { $($inner:tt)* } $($rest:tt)*) => {
        $output.extend(std::iter::once(TokenTree::Ident(Ident::new("unsafe", proc_macro::Span::call_site()))));
        let mut inner = TokenStream::new();
        quote_inner!(inner, $($inner)*);
        $output.extend(std::iter::once(TokenTree::Group(Group::new(Delimiter::Brace, inner))));
        quote_inner!($output, $($rest)*);
    };
    ($output:ident, { $($inner:tt)* } $($rest:tt)*) => {
        let mut inner = TokenStream::new();
        quote_inner!(inner, $($inner)*);
        $output.extend(std::iter::once(TokenTree::Group(Group::new(Delimiter::Brace, inner))));
        quote_inner!($output, $($rest)*);
    };
    ($output:ident, ( $($inner:tt)+ ) $($rest:tt)*) => {
        let mut inner = TokenStream::new();
        quote_inner!(inner, $($inner)+);
        $output.extend(std::iter::once(TokenTree::Group(Group::new(Delimiter::Parenthesis, inner))));
        quote_inner!($output, $($rest)*);
    };
    ($output:ident, $i:ident : $($rest:tt)*) => {
        $output.extend(std::iter::once(TokenTree::Ident(Ident::new(stringify!($i), proc_macro::Span::call_site()))));
        $output.extend(std::iter::once(TokenTree::Punct(Punct::new(':', Spacing::Alone))));
        quote_inner!($output, $($rest)*);
    };
    ($output:ident, * $($rest:tt)*) => {
        $output.extend(std::iter::once(TokenTree::Punct(Punct::new('*', Spacing::Alone))));
        quote_inner!($output, $($rest)*);
    };
    ($output:ident, $func:ident ( $($args:tt)* ) $($rest:tt)*) => {
        $output.extend(std::iter::once(TokenTree::Ident(Ident::new(stringify!($func), proc_macro::Span::call_site()))));
        let mut inner = TokenStream::new();
        quote_inner!(inner, $($args)*);
        $output.extend(std::iter::once(TokenTree::Group(Group::new(Delimiter::Parenthesis, inner))));
        quote_inner!($output, $($rest)*);
    };
    ($output:ident, $i:ident $($rest:tt)*) => {
        $output.extend(std::iter::once(TokenTree::Ident(Ident::new(stringify!($i), proc_macro::Span::call_site()))));
        quote_inner!($output, $($rest)*);
    };
    ($output:ident, $lit:literal $($rest:tt)*) => {{
        let literal = match stringify!($lit) {
            s if s.starts_with('"') => Literal::string(&s[1..s.len()-1]),
            s => Literal::isize_unsuffixed(s.parse().unwrap()),
        };
        $output.extend(std::iter::once(TokenTree::Literal(literal)));
        quote_inner!($output, $($rest)*);
    }};
    ($output:ident, $punct:tt $($rest:tt)*) => {
        quote_punct!($output, $punct);
        quote_inner!($output, $($rest)*);
    };
    ($output:ident, $lit:literal $($rest:tt)*) => {
        $output.extend(std::iter::once(TokenTree::Literal(Literal::string(stringify!($lit)))));
        quote_inner!($output, $($rest)*);
    };
    ($output:ident, ( $($inner:tt)* ) $($rest:tt)*) => {
        let mut inner = TokenStream::new();
        quote_inner!(inner, $($inner)*);
        $output.extend(std::iter::once(TokenTree::Group(Group::new(Delimiter::Parenthesis, inner))));
        quote_inner!($output, $($rest)*);
    };
    ($output:ident, [ $($inner:tt)* ] $($rest:tt)*) => {
        let mut inner = TokenStream::new();
        quote_inner!(inner, $($inner)*);
        $output.extend(std::iter::once(TokenTree::Group(Group::new(Delimiter::Bracket, inner))));
        quote_inner!($output, $($rest)*);
    };
    ($output:ident,) => {};
}

macro_rules! quote_punct {
    ($output:ident, =) => {
        $output.extend(std::iter::once(TokenTree::Punct(Punct::new('=', Spacing::Alone))));
    };
    ($output:ident, ;) => {
        $output.extend(std::iter::once(TokenTree::Punct(Punct::new(';', Spacing::Alone))));
    };
    ($output:ident, ?) => {
        $output.extend(std::iter::once(TokenTree::Punct(Punct::new('?', Spacing::Alone))));
    };
    ($output:ident, .) => {
        $output.extend(std::iter::once(TokenTree::Punct(Punct::new('.', Spacing::Alone))));
    };
    ($output:ident, ::) => {
        $output.extend(vec![TokenTree::Punct(Punct::new(':', Spacing::Joint)), TokenTree::Punct(Punct::new(':', Spacing::Alone))]);
    };
    ($output:ident, $other:tt) => {
        $output.extend(std::iter::once(TokenTree::Punct(Punct::new(stringify!($other).chars().next().unwrap(), Spacing::Alone))));
    };
}
