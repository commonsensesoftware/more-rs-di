use super::{CallSite, InjectedCallSite};
use proc_macro2::{Ident, Span};
use syn::{spanned::Spanned, Error, FnArg, ImplItem, ItemImpl, Path, Result, Signature};

pub struct Constructor;

impl Constructor {
    pub fn select<'a>(impl_: &'a ItemImpl, path: &Path) -> Result<&'a Signature> {
        let new = Ident::new("new", Span::call_site());
        let mut convention = Option::None;
        let mut methods = Vec::new();

        for item in &impl_.items {
            if let ImplItem::Fn(method) = item {
                let signature = &method.sig;

                if method.attrs.iter().any(|a| a.path().is_ident("inject")) {
                    methods.push(signature);
                }

                if signature.ident == new {
                    convention = Some(signature);
                }
            }
        }

        match methods.len() {
            0 => {
                if let Some(method) = convention {
                    Ok(method)
                } else {
                    Err(Error::new(
                        impl_.span(),
                        format!(
                            "Neither {}::new or an associated method decorated with #[inject] was found.",
                            path.segments.last().unwrap().ident
                        ),
                    ))
                }
            }
            1 => Ok(methods[0]),
            _ => Err(Error::new(
                impl_.span(),
                format!(
                    "{} has more than one associated method decorated with #[inject].",
                    path.segments.last().unwrap().ident
                ),
            )),
        }
    }

    pub fn visit(ctor: &Signature) -> Result<Vec<InjectedCallSite>> {
        let count = ctor.inputs.len();

        if count == 0 {
            return Ok(Vec::with_capacity(0));
        }

        let mut callsites = Vec::with_capacity(count);

        for input in ctor.inputs.iter() {
            let callsite = match input {
                FnArg::Typed(ref type_) => CallSite::visit(&*type_.ty, false)?,
                _ => {
                    return Err(Error::new(
                        input.span(),
                        "The argument must be ServiceRef, KeyedServiceRef, Rc, or \
                              Arc and optionally wrapped with Option or Vec.",
                    ))
                }
            };

            callsites.push(callsite);
        }

        Ok(callsites)
    }
}
