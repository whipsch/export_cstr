#![feature(core, rustc_private, plugin_registrar)]

extern crate rustc;
extern crate syntax;

use rustc::plugin::registry::Registry;
use syntax::ptr::P;
use syntax::ext::base;
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::codemap::Span;
use syntax::ast;
use syntax::ast::Visibility;
use syntax::ast::ItemStatic;
use syntax::ast::Sign::Plus;
use syntax::ast::Mutability::MutImmutable;
use syntax::ast::Lit_::{LitChar, LitInt, LitStr};
use syntax::ast::Ty_::TyFixedLengthVec;
use syntax::ast::LitIntType::UnsuffixedIntLit;
use syntax::parse::token;


/// Generate an `ast::Expr` for `'a' as i8`
fn make_char_cast<'cx>(cx: &'cx mut ExtCtxt, sp: Span, c: char) -> P<ast::Expr> {
    let path = cx.ty_path(cx.path_ident(sp, cx.ident_of("i8")));
    cx.expr_cast(sp, cx.expr_lit(sp, LitChar(c)), path)
}

/// Ensure `expr` is a string literal, and return either Some(&str) or None.
fn extract_literal<'cx, 'a>(cx: &'cx mut ExtCtxt, expr: &'a P<ast::Expr>) -> Option<&'a str> {
    if let ast::ExprLit(ref lit) = expr.node {
        if let ast::LitStr(ref s, _) = lit.node {
            Some(s.get())
        } else {
            cx.span_err(expr.span, "expected a string literal");
            None
        }
    } else {
        cx.span_err(expr.span, "expected a string literal");
        None
    }
}

/// Construct an attribute #[foo]
fn make_attr_word<'cx>(cx: &'cx mut ExtCtxt, sp: Span, name: &str) -> ast::Attribute {
    cx.attribute(sp, cx.meta_word(sp, token::intern_and_get_ident(name)))
}

/// Construct an attribute #[foo(bar)] or #[baz(biff, quux, ...)]
fn make_attr_list<'cx>(cx: &'cx mut ExtCtxt, sp: Span, name: &str, list: Vec<&str>) -> ast::Attribute { 
   let words = list.iter().map(|w| { cx.meta_word(sp, token::intern_and_get_ident(w)) }).collect::<Vec<_>>();
   cx.attribute(sp, cx.meta_list(sp, token::intern_and_get_ident(name), words))
}

/// Expansion for `declare_static_raw_cstr!`
/// `declare_static_raw_cstr!("foo", "hello")` expands to:
///
/// `#[no_mangle] #[allow(dead_code, non_upper_case_globals)]
/// pub static foo: [i8; 6] = ['h' as i8, 'e' as i8, ..., 0 as i8];`
fn expand_declare_static_raw_cstr<'cx>(cx: &'cx mut ExtCtxt, sp: Span, tts: &[ast::TokenTree])
                                     -> Box<base::MacResult + 'cx> {
    let exprs = match base::get_exprs_from_tts(cx, sp, tts) {
        Some(e) => e,
        None => return base::DummyResult::expr(sp)
    };

    let mut it = exprs.iter();
    let e_name = if let Some(expr) = it.next() {
        extract_literal(cx, expr)
    } else {
        cx.span_err(sp, "expected 2 arguments, found 0");
        None
    };
    let e_lit = if let Some(expr) = it.next() {
        extract_literal(cx, expr)
    } else {
        cx.span_err(sp, "expected 2 arguments, found 1");
        None
    };

    match (e_name, e_lit, it.count()) {
        // name, literal, and no other arguments remaining
        (Some(name), Some(s), 0) => {
            // append null terminator to make it a cstring
            let lit = format!("{}\0", s);

            // build RHS of the statement
            let rhs = lit.as_slice().chars().map(|c| { make_char_cast(cx, sp, c) }).collect::<Vec<_>>();
            let rhs = cx.expr_vec(sp, rhs);

            // the type of the item is a fixed length vector of i8
            let path = cx.ty_path(cx.path_ident(sp, cx.ident_of("i8")));
            let ty = cx.ty(sp, TyFixedLengthVec(path, cx.expr_lit(sp, LitInt(lit.len() as u64, UnsuffixedIntLit(Plus)))));

            // #[no_mangle] #[allow(dead_code, non_upper_case_globals)]
            let attrs = vec![
                make_attr_word(cx, sp, "no_mangle"),
                make_attr_list(cx, sp, "allow", vec!["dead_code", "non_upper_case_globals"])
            ];

            // XXX: have to manually construct the item here because we can't set the visibility otherwise.
            let item = P(ast::Item {
                ident: cx.ident_of(name),
                attrs: attrs,
                id: ast::DUMMY_NODE_ID,
                node: ast::ItemStatic(ty, MutImmutable, rhs),
                vis: Visibility::Public,
                span: sp
            });
            base::MacItems::new(vec![item].into_iter())
        }
        (_, _, rest) => {
            if rest > 0 {
                cx.span_err(sp, format!("expected 2 arguments, found {}", rest + 2).as_slice());
            }
            base::DummyResult::expr(sp)
        }
    }
}

/// Declare a `pub static [i8; N]`, where N is the length of the supplied string, plus 1.
/// The resulting item is a raw, null-terminated C string that becomes exported as an unmangled
/// symbol for use in dynamic libraries.
///
/// `export_cstr!(foo, "hello")` expands to:
///
/// `#[no_mangle] #[allow(dead_code, non_upper_case_globals)]
/// pub static foo: [i8; 6] = ['h' as i8, 'e' as i8', ..., 0 as i8];`
#[macro_export]
macro_rules! export_cstr {
    ($name:ident, $lit:expr) => (
        declare_static_raw_cstr!(stringify!($name), $lit);
    );
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("declare_static_raw_cstr", expand_declare_static_raw_cstr);
}

