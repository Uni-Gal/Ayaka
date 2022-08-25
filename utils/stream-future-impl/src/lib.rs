use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, Parser},
    parse_macro_input,
    visit_mut::{visit_expr_mut, visit_stmt_mut, VisitMut},
    AttributeArgs, Block, Expr, ItemFn, Lifetime, Lit, Meta, NestedMeta, ReturnType, Type,
};

#[proc_macro_attribute]
pub fn stream(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as AttributeArgs);
    let mut p_type = Type::parse.parse2(quote!(())).unwrap();
    let mut lifetime = Lifetime::parse.parse2(quote!('static)).unwrap();
    for a in attr {
        match a {
            NestedMeta::Meta(Meta::Path(path)) => {
                p_type = Type::parse.parse2(quote!(#path)).unwrap()
            }
            NestedMeta::Meta(Meta::NameValue(value)) => {
                if value
                    .path
                    .get_ident()
                    .map(|ident| ident.to_string())
                    .unwrap_or_default()
                    == "lifetime"
                {
                    if let Lit::Str(lit) = value.lit {
                        lifetime = Lifetime::parse.parse_str(&lit.value()).unwrap();
                    }
                }
            }
            NestedMeta::Lit(Lit::Str(lit)) => p_type = Type::parse.parse_str(&lit.value()).unwrap(),
            _ => unreachable!(),
        }
    }

    let func = input.clone();
    let mut func = parse_macro_input!(func as ItemFn);
    func.sig.asyncness = None;
    let future_return_type = match func.sig.output {
        ReturnType::Default => Box::new(Type::parse.parse2(quote!(())).unwrap()),
        ReturnType::Type(_, t) => t,
    };
    func.sig.output = ReturnType::parse
        .parse2(quote! {
            -> impl ::core::future::Future<Output = #future_return_type> + ::stream_future::Stream<Item = #p_type> + #lifetime
        })
        .unwrap();
    let mut old_block = func.block;
    for stmt in old_block.stmts.iter_mut() {
        visit_stmt_mut(&mut AwaitYieldVisitor, stmt);
    }
    func.block = Box::new(
        Block::parse
            .parse2(quote! {{
                ::stream_future::GenFuture::<#p_type, _>::new(static move |__cx: ::stream_future::ResumeTy| {
                    #old_block
                })
            }})
            .unwrap(),
    );

    func.to_token_stream().into()
}

struct AwaitYieldVisitor;

impl VisitMut for AwaitYieldVisitor {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        match i {
            Expr::Await(expr_await) => {
                let attrs = &expr_await.attrs;
                let mut inner_expr = expr_await.base.clone();
                self.visit_expr_mut(&mut inner_expr);
                *i = Expr::parse
                    .parse2(quote! {
                        #(#attrs)*
                        {
                            let mut __future = #inner_expr;
                            loop {
                                #[allow(unsafe_code)]
                                let mut __future = unsafe { ::core::pin::Pin::new_unchecked(&mut __future) };
                                match __cx.poll_future(__future) {
                                    ::core::task::Poll::Ready(__ret) => {
                                        break __ret;
                                    }
                                    ::core::task::Poll::Pending => {
                                        yield ::core::task::Poll::Pending;
                                    }
                                }
                            }
                        }
                    })
                    .unwrap();
            }
            _ => visit_expr_mut(self, i),
        }
    }

    fn visit_expr_yield_mut(&mut self, i: &mut syn::ExprYield) {
        let mut inner_expr = i
            .expr
            .take()
            .unwrap_or_else(|| Box::new(Expr::parse.parse2(quote!(())).unwrap()));
        self.visit_expr_mut(&mut inner_expr);
        i.expr = Some(Box::new(
            Expr::parse
                .parse2(quote!(::core::task::Poll::Ready(
                    #[allow(unused_parens)]
                    #inner_expr
                )))
                .unwrap(),
        ));
    }

    fn visit_expr_async_mut(&mut self, _i: &mut syn::ExprAsync) {}
}
