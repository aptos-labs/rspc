//! This comment contains an overview of the rationale behind the design of the middleware system.
//! NOTE: It is *not* included in the generated Rust docs!
//!
//! For future reference:
//!
//! Having a standalone middleware that is like `fn my_middleware() -> impl Middleware<...>` results in *really* bad error messages.
//! This is because the middleware is defined within the function and then *constrained* at the function boundary.
//! These places are different so the compiler is like lol trait xyz with generics iop does match the trait xyz with generics abc.
//!
//! Instead if the builder function takes a [`MiddlewareBuilder`] the constrain it applied prior to the middleware being defined.
//! This allows the compiler to constrain the types at the middleware definition site which leads to insanely better error messages.
//!
//! Be aware this talk about constraining and definition is just me speaking about what I have observed.
//! This could be completely wrong from a technical perspective but we can all agree better errors big good.
//!
//! TODO: Explaining why inference across boundaries is not supported.
//!
//! TODO: Explain why we can't have `fn mw(...) -> Middleware` -> It's because of default generics!!!
//!
//! TODO: Why we can't use `const`'s for declaring middleware -> Boxing

use std::{any::Any, error, marker::PhantomData, pin::Pin, sync::Arc};

use futures::Future;

use crate::{procedure::ProcedureMeta, State};

use super::{next::NextInner, Next};

pub(crate) type MiddlewareHandler<TNextCtx, TNextInput, TNextResult> = Box<
    dyn Fn(
            TNextCtx,
            TNextInput,
            ProcedureMeta,
        ) -> Pin<Box<dyn Future<Output = TNextResult> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

pub(crate) struct MiddlewareInner<TNextCtx, TNextInput, TNextResult> {
    pub(crate) setup: Option<Box<dyn FnOnce(&mut State, ProcedureMeta) + 'static>>,
    pub(crate) handler: MiddlewareHandler<TNextCtx, TNextInput, TNextResult>,
}

/// An abstraction for common logic that can be applied to procedures.
///
/// A middleware can be used to run custom logic and modify the context, input, and result of the next procedure. This makes is perfect for logging, authentication and many other things!
///
/// Middleware are applied with [ProcedureBuilder::with](crate::procedure::ProcedureBuilder::with).
///
/// # Generics
///
/// - `TError` - The type of the error that can be returned by the middleware. Defined by [ProcedureBuilder::error](crate::procedure::ProcedureBuilder::error).
/// - `TThisCtx` - // TODO
/// - `TThisInput` - // TODO
/// - `TThisResult` - // TODO
/// - `TNextCtx` - // TODO
/// - `TNextInput` - // TODO
/// - `TNextResult` - // TODO
///
/// ```rust
/// TODO: Example to show where the generics line up.
/// ```
///
/// # Stacking
///
/// TODO: Guide the user through stacking.
///
/// # Example
///
/// TODO:
///
// TODO: Explain why they are required -> inference not supported across boundaries.
pub struct Middleware<
    TError: error::Error,
    TThisCtx,
    TThisInput,
    TThisResult,
    TNextCtx = TThisCtx,
    TNextInput = TThisInput,
    TNextResult = TThisResult,
> {
    // TODO: Move the builder `Fn` down onto `handler` without breaking everything!
    pub(crate) inner: Box<
        dyn FnOnce(
            Arc<(dyn Any + Send + Sync + 'static)>,
        ) -> MiddlewareInner<TThisCtx, TThisInput, TThisResult>,
    >,
    phantom: PhantomData<(TError, TNextCtx, TNextInput, TNextResult)>,
}

// TODO: Debug impl

impl<TError, TThisCtx, TThisInput, TThisResult, TNextCtx, TNextInput, TNextResult>
    Middleware<TError, TThisCtx, TThisInput, TThisResult, TNextCtx, TNextInput, TNextResult>
where
    TError: error::Error,
{
    // TODO: Allow returning results with `TErr`
    pub fn new<F: Future<Output = TThisResult> + Send + 'static>(
        func: impl Fn(TThisCtx, TThisInput, Next<TNextCtx, TNextInput, TNextResult>) -> F
            + Send
            + Sync
            + 'static,
    ) -> Self {
        Self {
            inner: Box::new(move |next| MiddlewareInner {
                setup: None,
                handler: Box::new(move |ctx, input, meta| {
                    let f = func(
                        ctx,
                        input,
                        Next {
                            inner: NextInner {
                                meta,
                                next: next.clone(),
                            },
                            phantom: PhantomData,
                        },
                    );

                    Box::pin(f)
                }),
            }),
            phantom: PhantomData,
        }
    }

    pub fn setup(mut self, func: impl FnOnce(&mut State, ProcedureMeta) + 'static) -> Self {
        // self.inner.setup = Some(Box::new(func));
        todo!();
        self
    }
}