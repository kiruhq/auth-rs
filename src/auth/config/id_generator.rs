use super::ModelName;
use ksuid::Ksuid;

pub trait IdGenerator: Send + Sync + 'static {
    fn generate(&self, model_name: ModelName) -> String;
}

pub(super) struct KsuidGenerator;

impl IdGenerator for KsuidGenerator {
    fn generate(&self, model_name: super::ModelName) -> String {
        let ksuid = Ksuid::generate();
        format!("{}_{}", model_name.to_string(), ksuid.to_base62())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_valid_ksuid_id(id: &str, prefix: &str) {
        assert!(id.starts_with(prefix), "{id} should start with {prefix}");

        let suffix = &id[prefix.len()..];
        assert_eq!(suffix.len(), 27);
        Ksuid::from_base62(suffix).expect("generated id should contain a valid ksuid suffix");
    }

    #[test]
    fn prefixes_generated_ids_with_model_name() {
        let generator = KsuidGenerator;

        assert_valid_ksuid_id(&generator.generate(ModelName::User), "user_");
        assert_valid_ksuid_id(&generator.generate(ModelName::Account), "account_");
        assert_valid_ksuid_id(&generator.generate(ModelName::Session), "session_");
        assert_valid_ksuid_id(
            &generator.generate(ModelName::Verification),
            "verification_",
        );
    }

    #[test]
    fn prefixes_custom_model_ids_with_custom_name() {
        let generator = KsuidGenerator;

        assert_valid_ksuid_id(&generator.generate(ModelName::Custom("team")), "team_");
    }

    #[test]
    fn generates_distinct_ids_for_the_same_model() {
        let generator = KsuidGenerator;

        let first = generator.generate(ModelName::User);
        let second = generator.generate(ModelName::User);

        assert_ne!(first, second);
    }
}
