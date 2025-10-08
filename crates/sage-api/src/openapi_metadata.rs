/// Metadata for `OpenAPI` endpoint documentation
///
/// This trait is implemented by request types to provide their `OpenAPI` category/tag
/// and other documentation metadata.
pub trait OpenApiMetadata {
    /// The `OpenAPI` tag/category for this endpoint (e.g., "Authentication & Keys")
    fn openapi_tag() -> &'static str;

    /// Optional: More detailed description beyond the type's doc comment
    fn openapi_description() -> Option<&'static str> {
        None
    }
}
