//! Configuration Module
//!
//! Provides file-based configuration for mcb-validate, allowing
//! projects to customize validation rules via `.mcb-validate.toml`.

mod file_config;

pub use file_config::{
    ArchitectureRulesConfig, FileConfig, GeneralConfig, OrganizationRulesConfig,
    QualityRulesConfig, RulesConfig, ShakuRulesConfig, SolidRulesConfig, ValidatorsConfig,
};
