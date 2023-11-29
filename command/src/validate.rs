//(language,message)
pub(crate) fn validate_language_flag<'s>(language: &'s str) -> (&'s str, &'static str) {
    if language == "Dockerfile" {
        (
            "dockerfile",
            r#"language "Dockerfile" was converted to "dockerfile" automatically"#,
        )
    } else {
        (language, "")
    }
}
