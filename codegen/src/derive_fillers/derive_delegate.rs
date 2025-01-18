use portrait_framework::{DeriveContext, GenerateDerive, NoArgs};
use proc_macro2::Span;
use syn::spanned::Spanned;

use crate::util;

pub(crate) struct Generator(pub(crate) NoArgs);

impl GenerateDerive for Generator {
    fn generate_const(
        &mut self,
        _ctx: DeriveContext,
        item: &syn::TraitItemConst,
    ) -> syn::Result<syn::ImplItemConst> {
        Err(syn::Error::new_spanned(item, "derive_delegate does not support const items"))
    }

    fn generate_fn(
        &mut self,
        DeriveContext { input, trait_path, .. }: DeriveContext,
        item: &syn::TraitItemFn,
    ) -> syn::Result<syn::ImplItemFn> {
        let fn_args = util::parse_grouped_attr::<FnArgs>(&item.attrs, "derive_delegate")?;

        let output_ty: syn::Type = if let syn::ReturnType::Type(_, ty) = &item.sig.output {
            if fn_args.with_try.0.is_some() {
                let make_err = || {
                    syn::Error::new_spanned(
                        ty,
                        "`with_try` must be used with a type in the form `R<T, ...>` where `R` is \
                         a Try type e.g. `Result`/`Option`.",
                    )
                };

                let syn::Type::Path(path) = &**ty else {
                    return Err(make_err());
                };
                let last = path.path.segments.last().expect("path should not be empty");
                let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
                    return Err(make_err());
                };
                let Some(syn::GenericArgument::Type(ok_ty)) = args.args.first() else {
                    return Err(make_err());
                };
                ok_ty.clone()
            } else {
                (**ty).clone()
            }
        } else {
            syn::Type::Tuple(syn::parse_quote!(()))
        };

        let mut stmts = match &input.data {
            syn::Data::Struct(data) => {
                transform_struct(item, trait_path, &fn_args, &output_ty, data)?
            }
            syn::Data::Enum(data) => transform_enum(item, trait_path, &fn_args, &output_ty, data)?,
            syn::Data::Union(data) => {
                return Err(syn::Error::new_spanned(
                    data.union_token,
                    "derive_delegate does not support unions",
                ))
            }
        };

        if let &Some((with_try_span, ref with_try)) = &fn_args.with_try.0 {
            let try_fn = with_try
                .clone()
                .unwrap_or_else(|| syn::Expr::Path(syn::parse_quote_spanned!(with_try_span => Ok)));

            let old_stmt_block = syn::Block {
                brace_token: syn::token::Brace(with_try_span),
                stmts:       stmts.drain(..).collect(),
            };

            let wrapped_stmt = syn::Expr::Call(syn::ExprCall {
                attrs:       Vec::new(),
                func:        Box::new(try_fn),
                paren_token: syn::token::Paren(with_try_span),
                args:        [syn::Expr::Block(syn::ExprBlock {
                    attrs: Vec::new(),
                    label: None,
                    block: old_stmt_block,
                })]
                .into_iter()
                .collect(),
            });

            stmts.push(syn::Stmt::Expr(wrapped_stmt, None));
        }

        Ok(syn::ImplItemFn {
            attrs:       item
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("cfg"))
                .cloned()
                .collect(),
            vis:         syn::Visibility::Inherited,
            defaultness: None,
            sig:         item.sig.clone(),
            block:       syn::Block { brace_token: Default::default(), stmts },
        })
    }

    fn generate_type(
        &mut self,
        _ctx: DeriveContext,
        item: &syn::TraitItemType,
    ) -> syn::Result<syn::ImplItemType> {
        Err(syn::Error::new_spanned(item, "derive_delegate does not support type items"))
    }

    fn extend_generics(
        &mut self,
        DeriveContext { trait_path, input, .. }: DeriveContext,
        _generics_params: &mut Vec<syn::GenericParam>,
        generics_where: &mut Vec<syn::WherePredicate>,
    ) -> syn::Result<()> {
        fn add_generic_predicate(
            generics_where: &mut Vec<syn::WherePredicate>,
            trait_path: &syn::Path,
            field: &syn::Field,
        ) {
            generics_where.push(syn::WherePredicate::Type(syn::PredicateType {
                lifetimes:   None,
                bounded_ty:  field.ty.clone(),
                colon_token: syn::Token![:](field.span()),
                bounds:      [syn::TypeParamBound::Trait(syn::TraitBound {
                    paren_token: None,
                    modifier:    syn::TraitBoundModifier::None,
                    lifetimes:   None,
                    path:        trait_path.clone(),
                })]
                .into_iter()
                .collect(),
            }));
        }

        match &input.data {
            syn::Data::Struct(data) => {
                for field in &data.fields {
                    add_generic_predicate(generics_where, trait_path, field);
                }
            }
            syn::Data::Enum(data) => {
                for variant in &data.variants {
                    for field in &variant.fields {
                        add_generic_predicate(generics_where, trait_path, field);
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}

fn transform_struct(
    item: &syn::TraitItemFn,
    trait_path: &syn::Path,
    fn_args: &FnArgs,
    output_ty: &syn::Type,
    data: &syn::DataStruct,
) -> syn::Result<Vec<syn::Stmt>> {
    let mut stmts = Vec::new();

    if let Some(receiver) = item.sig.receiver() {
        stmts.push(syn::Stmt::Local(syn::Local {
            attrs:      Vec::new(),
            let_token:  syn::Token![let](Span::call_site()),
            pat:        syn::Pat::Struct(syn::PatStruct {
                attrs:       Vec::new(),
                qself:       None,
                path:        syn::parse_quote!(Self),
                brace_token: syn::token::Brace(Span::call_site()),
                fields:      data
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(ord, field)| syn::FieldPat {
                        attrs:       cfg_attrs(&field.attrs),
                        member:      match &field.ident {
                            Some(ident) => syn::Member::Named(ident.clone()),
                            None => syn::Member::Unnamed(syn::Index {
                                index: u32::try_from(ord).expect("too many fields"),
                                span:  field.span(),
                            }),
                        },
                        colon_token: Some(syn::Token![:](field.span())),
                        pat:         Box::new(syn::Pat::Path(syn::ExprPath {
                            attrs: Vec::new(),
                            qself: None,
                            path:  syn::Path::from(quote::format_ident!("__portrait_self_{ord}")),
                        })),
                    })
                    .collect(),
                rest:        None,
            }),
            init:       Some(syn::LocalInit {
                eq_token: syn::Token![=](Span::call_site()),
                expr:     syn::parse_quote_spanned!(receiver.span() => self),
                diverge:  None,
            }),
            semi_token: syn::Token![;](Span::call_site()),
        }));
    }
    stmts.extend(transform_return(
        item,
        fn_args,
        trait_path,
        output_ty,
        &data.fields,
        &syn::parse_quote!(Self),
        false,
    )?);
    Ok(stmts)
}

fn transform_enum(
    item: &syn::TraitItemFn,
    trait_path: &syn::Path,
    fn_args: &FnArgs,
    output_ty: &syn::Type,
    data: &syn::DataEnum,
) -> syn::Result<Vec<syn::Stmt>> {
    let Some(receiver) = item.sig.receiver() else {
        return Err(syn::Error::new_spanned(
            &item.sig,
            "Cannot derive enum delegates for associated functions without receivers",
        ));
    };

    let mut arms = Vec::new();
    for variant in &data.variants {
        let variant_ident = &variant.ident;
        let arm_stmts = transform_return(
            item,
            fn_args,
            trait_path,
            output_ty,
            &variant.fields,
            &syn::parse_quote!(Self::#variant_ident),
            true,
        )?;

        let fields = variant
            .fields
            .iter()
            .enumerate()
            .map(|(ord, field)| syn::FieldPat {
                attrs:       cfg_attrs(&field.attrs),
                member:      match &field.ident {
                    Some(ident) => syn::Member::Named(ident.clone()),
                    None => syn::Member::Unnamed(syn::Index {
                        index: u32::try_from(ord).expect("too many fields"),
                        span:  field.span(),
                    }),
                },
                colon_token: Some(syn::Token![:](field.span())),
                pat:         Box::new(syn::Pat::Path(syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path:  syn::Path::from(quote::format_ident!("__portrait_self_{ord}")),
                })),
            })
            .collect();

        arms.push(syn::Arm {
            attrs:           cfg_attrs(&variant.attrs),
            pat:             syn::Pat::Struct(syn::PatStruct {
                attrs: Vec::new(),
                qself: None,
                path: syn::parse_quote!(Self::#variant_ident),
                brace_token: syn::token::Brace(variant.span()),
                fields,
                rest: Some(syn::PatRest {
                    attrs:      Vec::new(),
                    dot2_token: syn::Token![..](variant.span()),
                }),
            }),
            guard:           None,
            fat_arrow_token: syn::Token![=>](variant.span()),
            body:            Box::new(syn::Expr::Block(syn::ExprBlock {
                attrs: Vec::new(),
                label: None,
                block: syn::Block {
                    brace_token: syn::token::Brace(variant.span()),
                    stmts:       arm_stmts,
                },
            })),
            comma:           Some(syn::Token![,](variant.span())),
        })
    }

    let match_stmt = syn::Stmt::Expr(
        syn::Expr::Match(syn::ExprMatch {
            attrs: Vec::new(),
            match_token: syn::Token![match](Span::call_site()),
            expr: Box::new(syn::parse_quote_spanned!(receiver.span() => self)),
            brace_token: syn::token::Brace(Span::call_site()),
            arms,
        }),
        None,
    );
    Ok(vec![match_stmt])
}

fn transform_return(
    item: &syn::TraitItemFn,
    fn_args: &FnArgs,
    trait_path: &syn::Path,
    output_ty: &syn::Type,
    fields: &syn::Fields,
    ctor_path: &syn::Path,
    is_refutable: bool,
) -> syn::Result<Vec<syn::Stmt>> {
    let exprs = transform_arg_fields(item, fn_args, trait_path, fields, ctor_path, is_refutable)?;

    let exprs = match exprs.try_into() {
        Ok::<[_; 1], _>([(single, _, _)]) => return Ok(vec![syn::Stmt::Expr(single, None)]),
        Err(err) => err,
    };

    Ok(match (&fn_args.reduce.0, output_ty) {
        (Some((_, reduce_fn)), _) => {
            let mut exprs_iter = exprs.into_iter();

            let mut stack = if let Some((_, reduce_base)) = &fn_args.reduce_base.0 {
                reduce_base.clone()
            } else {
                let Some((first, _, _)) = exprs_iter.next() else {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "derive_delegate(reduce) is not applicable for empty structs",
                    ));
                };
                first
            };

            for (expr, _, field) in exprs_iter {
                stack = syn::parse_quote_spanned! { field.span() =>
                    (#reduce_fn)(#stack, #expr)
                };
            }

            vec![syn::Stmt::Expr(stack, None)]
        }
        (_, syn::Type::Tuple(tuple)) if tuple.elems.is_empty() => exprs
            .into_iter()
            .map(|(delegate, _ord, field)| {
                syn::Stmt::Expr(delegate, Some(syn::Token![;](field.span())))
            })
            .collect(),
        (_, syn::Type::Path(ty_path)) if ty_path.path.is_ident("Self") => {
            let expr = syn::Expr::Struct(syn::ExprStruct {
                attrs:       Vec::new(),
                qself:       None,
                path:        ctor_path.clone(),
                brace_token: syn::token::Brace(item.span()),
                fields:      {
                    exprs
                        .into_iter()
                        .map(|(delegate, ord, field)| syn::FieldValue {
                            attrs:       cfg_attrs(&field.attrs),
                            member:      match &field.ident {
                                Some(ident) => syn::Member::Named(ident.clone()),
                                None => syn::Member::Unnamed(syn::Index {
                                    index: u32::try_from(ord).expect("too many fields"),
                                    span:  field.span(),
                                }),
                            },
                            colon_token: Some(syn::Token![:](field.span())),
                            expr:        delegate,
                        })
                        .collect()
                },
                dot2_token:  None,
                rest:        None,
            });
            let stmt = syn::Stmt::Expr(expr, None);
            vec![stmt]
        }
        _ => {
            return Err(syn::Error::new_spanned(
                output_ty,
                "Cannot determine how to aggregate the return value. Supported return types are \
                    `()`, `Self` or arbitrary types with the `#[portrait(derive_delegate(reduce = \
                    _))]` attribute, or `Option<>`/`Result<>` wrapping them with \
                    `#[portrait(derive_delegate(with_try))]`.",
            ))
        }
    })
}

fn transform_arg_fields<'t>(
    item: &syn::TraitItemFn,
    fn_args: &FnArgs,
    trait_path: &syn::Path,
    fields: &'t syn::Fields,
    ctor_path: &syn::Path,
    is_refutable: bool,
) -> syn::Result<Vec<(syn::Expr, usize, &'t syn::Field)>> {
    fields
        .iter()
        .enumerate()
        .map(|(ord, field)| {
            let mut expr = syn::Expr::Call(syn::ExprCall {
                attrs:       Vec::new(),
                func:        Box::new({
                    let mut func = trait_path.clone();
                    func.segments.push(item.sig.ident.clone().into());
                    syn::Expr::Path(syn::ExprPath { attrs: Vec::new(), qself: None, path: func })
                }),
                paren_token: syn::token::Paren(field.span()),
                args:        item
                    .sig
                    .inputs
                    .iter()
                    .map(|arg| transform_arg(arg, field, ord, ctor_path, is_refutable))
                    .collect::<syn::Result<_>>()?,
            });

            if let Some((with_try_span, _)) = fn_args.with_try.0 {
                expr = syn::Expr::Try(syn::ExprTry {
                    attrs:          Vec::new(),
                    expr:           Box::new(expr),
                    question_token: syn::Token![?](with_try_span),
                })
            }

            Ok((expr, ord, field))
        })
        .collect()
}

fn transform_arg(
    arg: &syn::FnArg,
    field: &syn::Field,
    ord: usize,
    ctor_path: &syn::Path,
    is_refutable: bool,
) -> syn::Result<syn::Expr> {
    let field_ident = quote::format_ident!("__portrait_self_{ord}");

    let ret = match arg {
        syn::FnArg::Receiver(_) => syn::Expr::Path(syn::parse_quote!(#field_ident)),
        syn::FnArg::Typed(arg) if is_self_ty(&arg.ty) => {
            if is_refutable {
                return Err(syn::Error::new_spanned(
                    arg,
                    "Non-receiver Self parameters are only supported for structs",
                ));
            }

            let member = match &field.ident {
                Some(ident) => syn::Member::Named(ident.clone()),
                None => syn::Member::Unnamed(syn::Index {
                    index: u32::try_from(ord).expect("too many fields"),
                    span:  field.span(),
                }),
            };
            syn::Expr::Block(syn::parse_quote! {{
                let #ctor_path { #member: __portrait_other, .. } = self;
                __portrait_other
            }})
        }
        syn::FnArg::Typed(arg) => syn::Expr::Path(syn::ExprPath {
            attrs: Vec::new(),
            qself: None,
            path:  {
                let syn::Pat::Ident(ident) = &*arg.pat else {
                    return Err(syn::Error::new_spanned(
                        &arg.pat,
                        "Cannot derive delegate for traits with non-identifier-pattern parameters",
                    ));
                };
                ident.ident.clone().into()
            },
        }),
    };
    Ok(ret)
}

fn is_self_ty(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(ty) => ty.path.is_ident("Self"),
        syn::Type::Reference(ty) => is_self_ty(&ty.elem),
        _ => false,
    }
}

mod kw {
    syn::custom_keyword!(reduce);
    syn::custom_keyword!(reduce_base);
}

#[derive(Default)]
struct FnArgs {
    reduce:      util::Once<syn::Expr>,
    reduce_base: util::Once<syn::Expr>,
    with_try:    util::Once<Option<syn::Expr>>,
}

impl util::ParseArgs for FnArgs {
    fn parse_once(&mut self, input: syn::parse::ParseStream) -> syn::Result<()> {
        let lh = input.lookahead1();
        if lh.peek(kw::reduce) {
            let key: kw::reduce = input.parse()?;
            let _: syn::Token![=] = input.parse()?;
            self.reduce.set(input.parse()?, key.span())?;
        } else if lh.peek(kw::reduce_base) {
            let key: kw::reduce_base = input.parse()?;
            let _: syn::Token![=] = input.parse()?;
            self.reduce_base.set(input.parse()?, key.span())?;
        } else if lh.peek(syn::Token![try]) {
            let key: syn::Token![try] = input.parse()?;

            let ok_expr = if input.peek(syn::Token![=]) {
                let _: syn::Token![=] = input.parse()?;
                let expr: syn::Expr = input.parse()?;
                Some(expr)
            } else {
                None
            };

            self.with_try.set(ok_expr, key.span())?;
        } else {
            return Err(lh.error());
        }
        Ok(())
    }
}

fn cfg_attrs<'t>(attrs: impl IntoIterator<Item = &'t syn::Attribute>) -> Vec<syn::Attribute> {
    attrs.into_iter().filter(|attr| attr.path().is_ident("cfg")).cloned().collect()
}
