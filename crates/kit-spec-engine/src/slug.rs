use regex::{Regex, RegexBuilder};
use std::sync::OnceLock;

use crate::error::{Result, SpecEngineError};

const MODULE_SLUG_PATTERN: &str = r"^[a-z0-9][a-z0-9-]*$";

static MODULE_SLUG_RE: OnceLock<Regex> = OnceLock::new();

pub(crate) fn validate_module_slug(slug: &str) -> Result<()> {
    if module_slug_re().is_match(slug) {
        Ok(())
    } else {
        Err(SpecEngineError::InvalidModuleSlug {
            slug: slug.to_owned(),
        })
    }
}

pub(crate) fn slug_boundary_regex(slug: &str) -> Result<Regex> {
    validate_module_slug(slug)?;
    RegexBuilder::new(&format!(
        r"(^|[^a-z0-9-]){}([^a-z0-9-]|$)",
        regex::escape(slug)
    ))
    .case_insensitive(true)
    .build()
    .map_err(|err| SpecEngineError::ParallelSchemaValidation {
        messages: err.to_string(),
    })
}

fn module_slug_re() -> &'static Regex {
    MODULE_SLUG_RE.get_or_init(|| Regex::new(MODULE_SLUG_PATTERN).expect("valid slug regex"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_expected_slugs() {
        for slug in ["a", "auth", "auth-service", "a1-b2"] {
            validate_module_slug(slug).expect(slug);
        }

        for slug in ["", "-auth", "Auth", "auth_service", "../auth"] {
            assert!(validate_module_slug(slug).is_err(), "{slug}");
        }
    }

    #[test]
    fn boundary_regex_does_not_match_inside_larger_slug() {
        let re = slug_boundary_regex("auth").unwrap();
        assert!(re.is_match("auth, session-store"));
        assert!(!re.is_match("oauth"));
        assert!(!re.is_match("authz"));
        assert!(!re.is_match("pre-auth-service"));
    }
}
