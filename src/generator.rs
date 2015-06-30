use syntax::abi;
use syntax::ast;
use syntax::ast_util;
use syntax::codemap::DUMMY_SP;
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;

use block::{Block, Describe, It, Bench};

pub trait Generate {
    fn generate(self, cx: &mut ExtCtxt, up: Option<&Describe>) -> P<ast::Item>;
}

impl Generate for Block {
    fn generate(self, cx: &mut ExtCtxt, up: Option<&Describe>) -> P<ast::Item> {
        match self {
            Block::Describe(describe) => describe.generate(cx, up),
            Block::It(it) => it.generate(cx, up),
            Block::Bench(bench) => bench.generate(cx, up)
        }
    }
}

impl Generate for Describe {
    fn generate(mut self,
                cx: &mut ExtCtxt,
                up: Option<&Describe>) -> P<ast::Item> {
        let name = cx.ident_of(&self.name);

        if let Some(ref up) = up {
            if let Some(ref before) = up.before {
                self.before = match self.before {
                    Some(ref now) => Some(merge_blocks(before, now)),
                    None => Some(before.clone())
                }
            }

            if let Some(ref after) = up.after {
                self.after = match self.after {
                    Some(ref now) => Some(merge_blocks(now, after)),
                    None => Some(after.clone())
                }
            }
        }

        let mut items = self.blocks.iter().map(|block| {
            block.clone().generate(cx, Some(&self))
        }).collect::<Vec<_>>();

        items.push(quote_item!(cx, pub use super::*;).unwrap());

        cx.item_mod(DUMMY_SP,
                    DUMMY_SP,
                    name,
                    vec![],
                    items)
    }
}

impl Generate for It {
    fn generate(self, cx: &mut ExtCtxt, up: Option<&Describe>) -> P<ast::Item> {
        let name = cx.ident_of(&self.name);
        let attrs = vec![quote_attr!(cx, #[test])];

        let block = if let Some(ref up) = up {
            match (&up.before, &up.after) {
                (&Some(ref before), &Some(ref after)) => {
                    merge_blocks(&merge_blocks(before, &self.block), after)
                },
                (&Some(ref before), &None) => merge_blocks(before, &self.block),
                (&None, &Some(ref after)) => merge_blocks(&self.block, after),
                (&None, &None) => self.block.clone()
            }
        } else {
            self.block
        };

        cx.item(DUMMY_SP, name, attrs,
                ast::ItemFn(
                    P(ast::FnDecl {
                        inputs: vec![],
                        output: ast::DefaultReturn(DUMMY_SP),
                        variadic: false
                    }),
                    ast::Unsafety::Normal,
                    ast::Constness::NotConst,
                    abi::Rust,
                    ast_util::empty_generics(),
                    block
                ))
    }
}

impl Generate for Bench {
    fn generate(self, cx: &mut ExtCtxt, up: Option<&Describe>) -> P<ast::Item> {
        let name = cx.ident_of(&self.name);

        let attrs = vec![quote_attr!(cx, #[bench])];

        let args = vec![
            cx.arg(DUMMY_SP, self.ident, quote_ty!(cx, &mut ::test::Bencher))
        ];

        let block = if let Some(ref up) = up {
            match (&up.before, &up.after) {
                (&Some(ref before), &Some(ref after)) => {
                    merge_blocks(&merge_blocks(before, &self.block), after)
                },
                (&Some(ref before), &None) => merge_blocks(before, &self.block),
                (&None, &Some(ref after)) => merge_blocks(&self.block, after),
                (&None, &None) => self.block.clone()
            }
        } else {
            self.block
        };

        cx.item(DUMMY_SP, name, attrs,
                ast::ItemFn(
                    P(ast::FnDecl {
                        inputs: args,
                        output: ast::DefaultReturn(DUMMY_SP),
                        variadic: false
                    }),
                    ast::Unsafety::Normal,
                    ast::Constness::NotConst,
                    abi::Rust,
                    ast_util::empty_generics(),
                    block
                ))
    }
}

fn merge_blocks(left: &P<ast::Block>, right: &P<ast::Block>) -> P<ast::Block> {
    use std::ops::Deref;

    let mut stmts = left.stmts.clone();
    stmts.extend(right.stmts.clone());

    P(ast::Block {
        stmts: stmts,
        ..left.deref().clone()
    })
}
