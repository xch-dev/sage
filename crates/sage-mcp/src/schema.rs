use std::collections::HashMap;

use serde::Serialize;
use serde_json::{Map, Value};
use utoipa::ToSchema;

const COMPONENT_REF_PREFIX: &str = "#/components/schemas/";

/// Builds a self-contained JSON schema for `T` from its `OpenAPI` definition.
pub(crate) fn schema_of<T: ToSchema>() -> Value {
    let mut collected = Vec::new();
    T::schemas(&mut collected);

    let components: HashMap<String, Value> = collected
        .into_iter()
        .map(|(name, schema)| (name, to_json(&schema)))
        .collect();

    let mut schema = to_json(&T::schema());
    inline_refs(&mut schema, &components, &mut Vec::new());
    schema
}

fn to_json(schema: &impl Serialize) -> Value {
    serde_json::to_value(schema).expect("OpenAPI schemas serialize to JSON")
}

/// Replaces component `$ref`s with their definitions, keeping sibling keys; cycles stay as refs.
fn inline_refs(value: &mut Value, components: &HashMap<String, Value>, stack: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if let Some(name) = component_ref(map)
                && !stack.contains(&name)
                && let Some(Value::Object(definition)) = components.get(&name)
            {
                let mut resolved = definition.clone();
                map.remove("$ref");
                resolved.extend(std::mem::take(map));

                let mut resolved = Value::Object(resolved);
                stack.push(name);
                inline_refs(&mut resolved, components, stack);
                stack.pop();

                *value = resolved;
                return;
            }

            for child in map.values_mut() {
                inline_refs(child, components, stack);
            }
        }
        Value::Array(items) => {
            for item in items {
                inline_refs(item, components, stack);
            }
        }
        _ => {}
    }
}

fn component_ref(map: &Map<String, Value>) -> Option<String> {
    map.get("$ref")?
        .as_str()?
        .strip_prefix(COMPONENT_REF_PREFIX)
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn components(entries: &[(&str, Value)]) -> HashMap<String, Value> {
        entries
            .iter()
            .map(|(name, schema)| ((*name).to_string(), schema.clone()))
            .collect()
    }

    #[test]
    fn ref_is_inlined_with_sibling_keys() {
        let components = components(&[("Mode", json!({ "type": "string", "enum": ["a"] }))]);
        let mut schema = json!({
            "type": "object",
            "properties": {
                "mode": { "$ref": "#/components/schemas/Mode", "description": "Mode" }
            }
        });

        inline_refs(&mut schema, &components, &mut Vec::new());

        assert_eq!(
            schema["properties"]["mode"],
            json!({ "type": "string", "enum": ["a"], "description": "Mode" })
        );
    }

    #[test]
    fn nested_refs_inside_arrays_are_inlined() {
        let components = components(&[
            ("Inner", json!({ "type": "string" })),
            (
                "Outer",
                json!({ "oneOf": [{ "type": "null" }, { "$ref": "#/components/schemas/Inner" }] }),
            ),
        ]);
        let mut schema = json!({ "$ref": "#/components/schemas/Outer" });

        inline_refs(&mut schema, &components, &mut Vec::new());

        assert_eq!(
            schema,
            json!({ "oneOf": [{ "type": "null" }, { "type": "string" }] })
        );
    }

    #[test]
    fn cyclic_ref_is_left_in_place() {
        let components = components(&[(
            "Node",
            json!({
                "type": "object",
                "properties": { "next": { "$ref": "#/components/schemas/Node" } }
            }),
        )]);
        let mut schema = json!({ "$ref": "#/components/schemas/Node" });

        inline_refs(&mut schema, &components, &mut Vec::new());

        assert_eq!(
            schema["properties"]["next"],
            json!({ "$ref": "#/components/schemas/Node" })
        );
    }

    #[test]
    fn unknown_ref_is_left_in_place() {
        let mut schema = json!({ "$ref": "#/components/schemas/Missing" });

        inline_refs(&mut schema, &HashMap::new(), &mut Vec::new());

        assert_eq!(schema, json!({ "$ref": "#/components/schemas/Missing" }));
    }
}
