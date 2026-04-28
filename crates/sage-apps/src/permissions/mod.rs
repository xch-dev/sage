mod capabilities;

pub(crate) use capabilities::{
    CapabilityFlags, CapabilityDefinition,
    get_system_capability_definition, get_user_capability_definition, user_registry,
};
