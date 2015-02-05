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
use syntax::ast::ItemStatic;
use syntax::ast::Sign::Plus;
use syntax::ast::Mutability::MutImmutable;
use syntax::ast::Lit_::{LitChar, LitInt};
use syntax::ast::Ty_::TyFixedLengthVec;
use syntax::ast::LitIntType::UnsuffixedIntLit;
use syntax::parse::token;


/// Construct a `ast::Ty_::TyPath` corresponding to `path`
fn make_ty_path<'cx>(cx: &'cx mut ExtCtxt, sp: Span, path: Vec<&str>) -> P<ast::Ty> {
    let path = path.iter().map(|p| { cx.ident_of(p) }).collect::<Vec<_>>();
    cx.ty_path(cx.path(sp, path))
}

/// Generate an `ast::Expr` for `'a' as libc::c_char`
fn make_char_cast<'cx>(cx: &'cx mut ExtCtxt, sp: Span, c: char) -> P<ast::Expr> {
    let path = make_ty_path(cx, sp, vec!["libc", "c_char"]);
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

/// Expansion for `declare_static_raw_cstr!`
/// `declare_static_raw_cstr!("foo", "hello")` -> `static foo: [libc::c_char; 6] = ['h' as libc::c_char, ..., 0 as libc::c_char];`
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

            // the type of the item is a fixed length vector of libc::c_char
            let path = make_ty_path(cx, sp, vec!["libc", "c_char"]);
            let ty = cx.ty(sp, TyFixedLengthVec(path, cx.expr_lit(sp, LitInt(lit.len() as u64, UnsuffixedIntLit(Plus)))));

            // make the actual item
            let attrs = vec![cx.attribute(sp, cx.meta_word(sp, token::intern_and_get_ident("no_mangle")))];
            let item = cx.item(sp, cx.ident_of(name), attrs, ast::ItemStatic(ty, MutImmutable, rhs));
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

/// Declare a `pub static [libc::c_char; N]`, where N is the length of the supplied string, plus 1.
/// The resulting item is a raw, null-terminated C string that becomes exported as an unmangled
/// symbol for use in dynamic libraries.
///
/// `export_cstr!(foo, "hello")` expands to `#[no_mangle] pub static foo: [libc::c_char; 6] = ['h' as libc::c_char, ..., 0 as libc::c_char];`
#[macro_export]
macro_rules! export_cstr {
    ($name:ident, $lit:expr) => (
        pub declare_static_raw_cstr!(stringify!($name), $lit);
    );
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("declare_static_raw_cstr", expand_declare_static_raw_cstr);
}

