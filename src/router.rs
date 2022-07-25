use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde_json::Value;
use specta::{to_ts, to_ts_export, DataType, EnumVariant, TypeDefs};

use crate::{ExecError, ExportError, OperationKey, OperationKind, Procedure, StreamOrValue};

pub struct Router<TCtx = (), TMeta = ()>
where
    TCtx: 'static,
{
    pub(crate) queries: HashMap<String, Procedure<TCtx>>,
    pub(crate) mutations: HashMap<String, Procedure<TCtx>>,
    pub(crate) subscriptions: HashMap<String, Procedure<TCtx>>,
    pub(crate) phantom: PhantomData<TMeta>,
}

impl<TCtx, TMeta> Router<TCtx, TMeta>
where
    TCtx: 'static,
{
    pub async fn exec(
        &self,
        ctx: TCtx,
        kind: OperationKind,
        key: OperationKey,
    ) -> Result<StreamOrValue, ExecError> {
        // TODO: This function should return either a stream or a value based on the `OperationKind` not an enum that could be both!
        // TODO: Reduce cloning in this function!
        match kind {
            OperationKind::Query => {
                (self
                    .queries
                    .get(&*key.0)
                    .ok_or(ExecError::OperationNotFound(key.0.clone()))?
                    .exec)(
                    ctx,
                    key.1.clone().unwrap_or(Value::Null),
                    (OperationKind::Query, key),
                )?
                .into_stream_or_value()
                .await
            }
            OperationKind::Mutation => {
                (self
                    .mutations
                    .get(&*key.0)
                    .ok_or(ExecError::OperationNotFound(key.0.clone()))?
                    .exec)(
                    ctx,
                    key.1.clone().unwrap_or(Value::Null),
                    (OperationKind::Mutation, key),
                )?
                .into_stream_or_value()
                .await
            }
            OperationKind::SubscriptionAdd => {
                (self
                    .subscriptions
                    .get(&*key.0)
                    .ok_or(ExecError::OperationNotFound(key.0.clone()))?
                    .exec)(
                    ctx,
                    key.1.clone().unwrap_or(Value::Null),
                    (OperationKind::SubscriptionAdd, key),
                )?
                .into_stream_or_value()
                .await
            }
            OperationKind::SubscriptionRemove => todo!(),
        }
    }

    pub fn arced(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub fn export_ts<TPath: AsRef<Path>>(&self, export_path: TPath) -> Result<(), ExportError> {
        let export_path = PathBuf::from(export_path.as_ref());
        fs::create_dir_all(&export_path)?;
        let mut file = File::create(export_path.clone().join("index.ts"))?;
        writeln!(file, "// This file was generated by [rspc](https://github.com/oscartbeaumont/rspc). Do not edit this file manually.")?;

        let queries_ts = generate_procedures_ts(&self.queries);
        let mutations_ts = generate_procedures_ts(&self.mutations);
        let subscriptions_ts = generate_procedures_ts(&self.subscriptions);

        writeln!(
            file,
            r#"
export type Operations = {{
    queries: {queries_ts},
    mutations: {mutations_ts},
    subscriptions: {subscriptions_ts}
}};"#
        )?;

        let mut defs_map = specta::TypeDefs::new();

        extract_procedures_types(&self.queries, &mut defs_map);
        extract_procedures_types(&self.mutations, &mut defs_map);
        extract_procedures_types(&self.subscriptions, &mut defs_map);

        println!("{:?}", defs_map);

        for export in defs_map.values().filter_map(|v| to_ts_export(v).ok()) {
            writeln!(file, "{}", export)?;
        }

        Ok(())
    }
}

fn generate_procedures_ts<Ctx>(procedures: &HashMap<String, Procedure<Ctx>>) -> String {
    match procedures.len() {
        0 => "never".to_string(),
        _ => procedures
            .iter()
            .map(|(key, subscription)| {
                let arg_ts = to_ts(&subscription.ty.arg_ty, false);
                let result_ts = to_ts(&subscription.ty.result_ty, false);

                format!(
                    r#"
        {{ key: "{key}", arg: {arg_ts}, result: {result_ts} }}"#
                )
            })
            .collect::<Vec<_>>()
            .join(" | "),
    }
}

fn extract_procedures_types<Ctx>(
    procedures: &HashMap<String, Procedure<Ctx>>,
    types_map: &mut TypeDefs,
) {
    for (_, procedure) in procedures {
        let arg_ty_defs = extract_types(&procedure.ty.arg_ty);
        types_map.extend(arg_ty_defs);

        let result_ty_defs = extract_types(&procedure.ty.result_ty);
        types_map.extend(result_ty_defs);
    }
}

fn extract_types(ty: &DataType) -> specta::TypeDefs {
    let mut defs = TypeDefs::new();

    match ty {
        DataType::Object(obj) => {
            if !obj.inline && !defs.contains_key(&obj.id) {
                defs.insert(obj.id, ty.clone());
            }

            obj.fields.iter().for_each(|field| {
                defs.extend(extract_types(&field.ty));
            });
        }
        DataType::Enum(e) => {
            if !e.inline && !defs.contains_key(&e.id) {
                defs.insert(e.id, ty.clone());
            }

            defs.extend(e.variants.iter().flat_map(|variant| match variant {
                EnumVariant::Unit(_) => TypeDefs::new(),
                EnumVariant::Unnamed(tuple) => extract_types(&DataType::Tuple(tuple.clone())),
                EnumVariant::Named(obj) => extract_types(&DataType::Object(obj.clone())),
            }));
        }
        DataType::List(ty) | DataType::Nullable(ty) => {
            defs.extend(extract_types(ty));
        }
        _ => {}
    }

    defs
}
