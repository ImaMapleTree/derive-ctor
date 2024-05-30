// while redundant this file exists as a possible user-reference to discover all possible properties
// available to them for a certain element


// error messages
pub(crate) const CONFIG_PROP_ERR_MSG: &str =
    "Unexpected property: \"{prop}\" (must be one of the following: \"{values}\")";
pub(crate) const DEFAULT_CTOR_ERR_MSG: &str =
    "Default constructor requires field to generate its own value.";

pub(crate) const CTOR_WORD: &str = "ctor";

// valid field properties
pub(crate) const FIELD_PROP_CLONED: &str = "cloned";
pub(crate) const FIELD_PROP_DEFAULT: &str = "default";
pub(crate) const FIELD_PROP_EXPR: &str = "expr";
pub(crate) const FIELD_PROP_INTO: &str = "into";
pub(crate) const FIELD_PROP_ITER: &str = "iter";

// valid enum-config properties
pub(crate) const ENUM_PROP_PREFIX: &str = "prefix";
pub(crate) const ENUM_PROP_VISIBILITY: &str = "visibility";
pub(crate) const ENUM_PROP_VIS: &str = "vis";

// variation property
pub(crate) const ENUM_VARIATION_PROP_NONE: &str = "none";

// struct config properties
pub(crate) const STRUCT_PROP_DEFAULT: &str = "default";
// property used within the default() prop
pub(crate) const NESTED_PROP_ALL: &str = "all";
