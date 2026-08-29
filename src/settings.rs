use clap::Parser;
use serde::Deserialize;
use std::{env, error::Error, fmt, fs, path::PathBuf, str::FromStr};

pub const WINDOW_LABEL: &str = "translation";

const DEFAULT_LANG: &str = "ru";
const DEFAULT_HOTKEYS: &str = "CTRL+SHIFT+F7";
const DEFAULT_GEMINI_MODEL: &str = "gemini-2.5-flash-lite";
const GOOGLE_TRANSLATE_URL: &str = "https://translate.googleapis.com/translate_a/single";
const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1";
const OPENAI_API_URL: &str = "https://api.openai.com/v1";

const ENV_SOURCE: &str = "EW_TRANSLATOR_SOURCE";
const ENV_API_KEY: &str = "EW_TRANSLATOR_API_KEY";
const ENV_MODEL: &str = "EW_TRANSLATOR_MODEL";
const ENV_API_URL: &str = "EW_TRANSLATOR_API_URL";
const ENV_LANG: &str = "EW_TRANSLATOR_LANG";
const ENV_HOTKEYS: &str = "EW_TRANSLATOR_HOTKEYS";
const ENV_CONFIG: &str = "EW_TRANSLATOR_CONFIG";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TranslationSource {
    GoogleTranslate,
    Gemini,
    OpenRouter,
    OpenAi,
}

impl TranslationSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GoogleTranslate => "google-translate",
            Self::Gemini => "gemini",
            Self::OpenRouter => "openrouter",
            Self::OpenAi => "openai",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::GoogleTranslate => "Google Translate",
            Self::Gemini => "Google Gemini",
            Self::OpenRouter => "OpenRouter",
            Self::OpenAi => "OpenAI-compatible API",
        }
    }
}

impl fmt::Display for TranslationSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for TranslationSource {
    type Err = SettingsError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().replace('_', "-").as_str() {
            "google" | "google-translate" => Ok(Self::GoogleTranslate),
            "gemini" | "google-api" | "google-gemini" => Ok(Self::Gemini),
            "openrouter" => Ok(Self::OpenRouter),
            "openai" => Ok(Self::OpenAi),
            value => Err(SettingsError::new(format!(
                "unknown translation source `{value}`; expected google-translate, gemini, openrouter, or openai"
            ))),
        }
    }
}

#[derive(Clone)]
pub struct TranslationSettings {
    pub source: TranslationSource,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub api_url: String,
}

#[derive(Clone)]
pub struct Settings {
    pub target_lang: String,
    pub hotkeys: String,
    pub translation: TranslationSettings,
}

impl Settings {
    pub fn print_summary(&self) {
        println!("Target language: `{}`", self.target_lang);
        println!("Hotkeys: `{}`", self.hotkeys);
        println!("Translation source: `{}`", self.translation.source);
        if let Some(model) = &self.translation.model {
            println!("Model: `{model}`");
        }
        println!("API URL: `{}`", self.translation.api_url);
    }
}

#[derive(Parser, Debug, Default)]
#[clap(
    author,
    version,
    about,
    long_about = None,
    after_help = "ENVIRONMENT:\n    EW_TRANSLATOR_SOURCE     google-translate (default), gemini, openrouter, or openai\n    EW_TRANSLATOR_API_KEY    API key for gemini, openrouter, or openai\n    EW_TRANSLATOR_MODEL      Model ID (required for openrouter and openai)\n    EW_TRANSLATOR_API_URL    Provider base URL; useful for OpenAI-compatible APIs\n    EW_TRANSLATOR_LANG       Target language [default: ru]\n    EW_TRANSLATOR_HOTKEYS    Global shortcut [default: CTRL+SHIFT+F7]\n    EW_TRANSLATOR_CONFIG     Path to an optional TOML config file\n\nPRECEDENCE:\n    Command-line options > environment variables > TOML config > defaults\n\nAPI keys intentionally have no command-line option to keep them out of shell history and process listings."
)]
struct Args {
    /// Target language (env: EW_TRANSLATOR_LANG)
    #[clap(short, long)]
    lang: Option<String>,

    /// Hotkeys (modifier+key; env: EW_TRANSLATOR_HOTKEYS)
    #[clap(short, long)]
    hotkeys: Option<String>,

    /// Translation source: google-translate, gemini, openrouter, or openai
    #[clap(long)]
    source: Option<String>,

    /// Model ID (env: EW_TRANSLATOR_MODEL)
    #[clap(long)]
    model: Option<String>,

    /// Provider base URL (env: EW_TRANSLATOR_API_URL)
    #[clap(long)]
    api_url: Option<String>,

    /// Optional TOML config path (env: EW_TRANSLATOR_CONFIG)
    #[clap(long, parse(from_os_str))]
    config: Option<PathBuf>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSettings {
    source: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
    api_url: Option<String>,
    lang: Option<String>,
    hotkeys: Option<String>,
}

impl From<&Args> for RawSettings {
    fn from(args: &Args) -> Self {
        Self {
            source: args.source.clone(),
            model: args.model.clone(),
            api_url: args.api_url.clone(),
            lang: args.lang.clone(),
            hotkeys: args.hotkeys.clone(),
            api_key: None,
        }
    }
}

#[derive(Debug)]
pub struct SettingsError(String);

impl SettingsError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for SettingsError {}

pub fn load() -> Result<Settings, SettingsError> {
    let args = Args::parse();
    let environment = read_environment();
    let (config_path, explicit_config) = select_config_path(&args);
    let config = read_config(config_path, explicit_config)?;
    let settings = resolve(RawSettings::from(&args), environment, config)?;
    settings.print_summary();
    Ok(settings)
}

fn read_environment() -> RawSettings {
    RawSettings {
        source: env::var(ENV_SOURCE).ok(),
        api_key: env::var(ENV_API_KEY).ok(),
        model: env::var(ENV_MODEL).ok(),
        api_url: env::var(ENV_API_URL).ok(),
        lang: env::var(ENV_LANG).ok(),
        hotkeys: env::var(ENV_HOTKEYS).ok(),
    }
}

fn select_config_path(args: &Args) -> (Option<PathBuf>, bool) {
    if let Some(path) = &args.config {
        return (Some(path.clone()), true);
    }
    if let Some(path) = env::var_os(ENV_CONFIG).filter(|path| !path.is_empty()) {
        return (Some(path.into()), true);
    }

    let base = env::var_os("XDG_CONFIG_HOME")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .filter(|path| !path.is_empty())
                .map(|path| PathBuf::from(path).join(".config"))
        });

    (
        base.map(|path| path.join("ew-translator/config.toml")),
        false,
    )
}

fn read_config(path: Option<PathBuf>, explicit: bool) -> Result<RawSettings, SettingsError> {
    let Some(path) = path else {
        return Ok(RawSettings::default());
    };

    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if !explicit && error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(RawSettings::default())
        }
        Err(error) => {
            return Err(SettingsError::new(format!(
                "cannot read config `{}`: {error}",
                path.display()
            )))
        }
    };

    toml::from_str(&contents).map_err(|error| {
        SettingsError::new(format!("invalid config `{}`: {error}", path.display()))
    })
}

fn resolve(
    command_line: RawSettings,
    environment: RawSettings,
    config: RawSettings,
) -> Result<Settings, SettingsError> {
    let source: TranslationSource = choose(
        command_line.source,
        environment.source,
        config.source,
        Some("google-translate".into()),
    )
    .expect("translation source has a default")
    .parse()?;
    let target_lang = required_value(
        "target language",
        choose(
            command_line.lang,
            environment.lang,
            config.lang,
            Some(DEFAULT_LANG.into()),
        ),
    )?;
    let hotkeys = required_value(
        "hotkeys",
        choose(
            command_line.hotkeys,
            environment.hotkeys,
            config.hotkeys,
            Some(DEFAULT_HOTKEYS.into()),
        ),
    )?;
    let api_key = optional_value(choose(
        command_line.api_key,
        environment.api_key,
        config.api_key,
        None,
    ));
    let model = optional_value(choose(
        command_line.model,
        environment.model,
        config.model,
        None,
    ));
    let api_url = required_value(
        "API URL",
        choose(
            command_line.api_url,
            environment.api_url,
            config.api_url,
            Some(default_api_url(source).into()),
        ),
    )?;

    validate_api_url(&api_url)?;
    match source {
        TranslationSource::GoogleTranslate => {}
        TranslationSource::Gemini => require_api_key(&api_key, source)?,
        TranslationSource::OpenRouter | TranslationSource::OpenAi => {
            require_api_key(&api_key, source)?;
            if model.is_none() {
                return Err(SettingsError::new(format!(
                    "{} requires a model; set {ENV_MODEL}, --model, or `model` in the config",
                    source.display_name()
                )));
            }
        }
    }

    let model = match source {
        TranslationSource::Gemini if model.is_none() => Some(DEFAULT_GEMINI_MODEL.into()),
        _ => model,
    };

    Ok(Settings {
        target_lang,
        hotkeys,
        translation: TranslationSettings {
            source,
            api_key,
            model,
            api_url: api_url.trim_end_matches('/').to_string(),
        },
    })
}

fn choose<T>(cli: Option<T>, env: Option<T>, config: Option<T>, default: Option<T>) -> Option<T> {
    cli.or(env).or(config).or(default)
}

fn optional_value(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

fn required_value(name: &str, value: Option<String>) -> Result<String, SettingsError> {
    optional_value(value).ok_or_else(|| SettingsError::new(format!("{name} cannot be empty")))
}

fn require_api_key(
    api_key: &Option<String>,
    source: TranslationSource,
) -> Result<(), SettingsError> {
    if api_key.is_some() {
        Ok(())
    } else {
        Err(SettingsError::new(format!(
            "{} requires an API key; set {ENV_API_KEY} or `api_key` in the config",
            source.display_name()
        )))
    }
}

fn default_api_url(source: TranslationSource) -> &'static str {
    match source {
        TranslationSource::GoogleTranslate => GOOGLE_TRANSLATE_URL,
        TranslationSource::Gemini => GEMINI_API_URL,
        TranslationSource::OpenRouter => OPENROUTER_API_URL,
        TranslationSource::OpenAi => OPENAI_API_URL,
    }
}

fn validate_api_url(value: &str) -> Result<(), SettingsError> {
    let url = reqwest::Url::parse(value)
        .map_err(|error| SettingsError::new(format!("invalid API URL `{value}`: {error}")))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(SettingsError::new(
            "API URL must use http or https (http is useful for local compatible servers)",
        ));
    }
    if url.host_str().is_none() {
        return Err(SettingsError::new("API URL must include a host"));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(SettingsError::new(
            "API URL must not contain credentials; use EW_TRANSLATOR_API_KEY",
        ));
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(SettingsError::new(
            "API URL must not contain a query string or fragment",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(source: &str) -> RawSettings {
        RawSettings {
            source: Some(source.into()),
            ..RawSettings::default()
        }
    }

    #[test]
    fn google_translate_is_the_default_without_credentials() {
        let settings = resolve(
            RawSettings::default(),
            RawSettings::default(),
            RawSettings::default(),
        )
        .unwrap();

        assert_eq!(
            settings.translation.source,
            TranslationSource::GoogleTranslate
        );
        assert_eq!(settings.target_lang, "ru");
        assert_eq!(settings.hotkeys, "CTRL+SHIFT+F7");
    }

    #[test]
    fn command_line_wins_over_environment_and_config() {
        let mut cli = raw("openai");
        cli.model = Some("cli-model".into());
        let mut environment = raw("openrouter");
        environment.api_key = Some("secret".into());
        environment.model = Some("env-model".into());
        let mut config = raw("gemini");
        config.api_key = Some("config-secret".into());
        config.model = Some("config-model".into());

        let settings = resolve(cli, environment, config).unwrap();

        assert_eq!(settings.translation.source, TranslationSource::OpenAi);
        assert_eq!(settings.translation.model.as_deref(), Some("cli-model"));
        assert_eq!(settings.translation.api_key.as_deref(), Some("secret"));
    }

    #[test]
    fn gemini_gets_a_stable_default_model() {
        let mut environment = raw("gemini");
        environment.api_key = Some("secret".into());

        let settings =
            resolve(RawSettings::default(), environment, RawSettings::default()).unwrap();

        assert_eq!(
            settings.translation.model.as_deref(),
            Some("gemini-2.5-flash-lite")
        );
    }

    #[test]
    fn openai_compatible_requires_key_and_model() {
        let missing_key = resolve(
            raw("openai"),
            RawSettings::default(),
            RawSettings::default(),
        )
        .err()
        .expect("OpenAI without an API key must fail");
        assert!(missing_key.to_string().contains(ENV_API_KEY));

        let mut environment = RawSettings {
            api_key: Some("secret".into()),
            ..RawSettings::default()
        };
        let missing_model = resolve(raw("openai"), environment.clone(), RawSettings::default())
            .err()
            .expect("OpenAI without a model must fail");
        assert!(missing_model.to_string().contains(ENV_MODEL));

        environment.model = Some("local-model".into());
        environment.api_url = Some("http://127.0.0.1:1234/v1".into());
        assert!(resolve(raw("openai"), environment, RawSettings::default()).is_ok());
    }

    #[test]
    fn rejects_secrets_embedded_in_api_url() {
        let cli = RawSettings {
            api_url: Some("https://secret@example.com/v1".into()),
            ..RawSettings::default()
        };

        let error = resolve(cli, RawSettings::default(), RawSettings::default())
            .err()
            .expect("credentials in an API URL must fail");

        assert!(error.to_string().contains("must not contain credentials"));
    }

    #[test]
    fn config_rejects_unknown_keys() {
        let error = toml::from_str::<RawSettings>("source = 'gemini'\ntoen = 'x'").unwrap_err();

        assert!(error.to_string().contains("unknown field"));
    }
}
