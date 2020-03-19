use crate::{
    registry, Context, ContextSelectionSet, ObjectType, OutputValueType, QueryError, Result, Type,
};
use graphql_parser::query::Field;
use serde_json::{Map, Value};
use std::borrow::Cow;

/// Empty mutation
///
/// Only the parameters used to construct the Schema, representing an unconfigured mutation.
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {}
///
/// fn main() {
///     let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
/// }
/// ```
pub struct EmptyMutation;

impl Type for EmptyMutation {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("EmptyMutation")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|_| registry::Type::Object {
            name: "EmptyMutation".to_string(),
            description: None,
            fields: Default::default(),
        })
    }
}

#[async_trait::async_trait]
impl ObjectType for EmptyMutation {
    fn is_empty() -> bool {
        true
    }

    async fn resolve_field(&self, _ctx: &Context<'_>, _name: &Field) -> Result<serde_json::Value> {
        unreachable!()
    }

    async fn resolve_inline_fragment(
        &self,
        _name: &str,
        _ctx: &ContextSelectionSet<'_>,
        _result: &mut Map<String, Value>,
    ) -> Result<()> {
        unreachable!()
    }
}

#[async_trait::async_trait]
impl OutputValueType for EmptyMutation {
    async fn resolve(_value: &Self, _ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        return Err(QueryError::NotConfiguredMutations.into());
    }
}
