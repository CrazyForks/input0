use crate::errors::AppError;
use crate::llm::client::LlmClient;
use crate::history;
use crate::config;

#[tauri::command]
pub async fn optimize_text(
    text: String,
    api_key: String,
    base_url: String,
    language: String,
) -> Result<String, AppError> {
    let client = LlmClient::new(api_key, base_url, None)?;
    let history = history::load_history();
    let config = config::load()?;
    let vocabulary = crate::vocabulary::load_vocabulary();

    let custom_active = config.custom_prompt_enabled && !config.custom_prompt.trim().is_empty();
    let clipboard = if custom_active && config.custom_prompt.contains("{{clipboard}}") {
        arboard::Clipboard::new()
            .and_then(|mut cb| cb.get_text())
            .ok()
    } else {
        None
    };

    let opts = crate::llm::client::OptimizeOptions {
        language: &language,
        history: &history,
        text_structuring: config.text_structuring,
        vocabulary: &vocabulary,
        source_app: None,
        user_tags: &config.user_tags,
        custom_prompt_enabled: config.custom_prompt_enabled,
        custom_prompt: &config.custom_prompt,
        clipboard: clipboard.as_deref(),
    };

    client.optimize_text_with_options(&text, &opts).await
}

#[tauri::command]
pub async fn test_api_connection(
    api_key: String,
    base_url: String,
    model: String,
) -> Result<String, AppError> {
    let model_opt = if model.is_empty() { None } else { Some(model) };
    let client = LlmClient::new(api_key, base_url, model_opt)?;
    client.test_connection().await
}

#[tauri::command]
pub async fn get_default_prompt_template(language: String) -> Result<String, AppError> {
    // Returns the built-in system prompt as a template, with `{{vocabulary}}`
    // and `{{user_tags}}` placeholders left in so the user can see which parts
    // get substituted at runtime. The safety rule is omitted because the
    // safety footer is auto-appended when the template is used.
    let config = config::load()?;
    Ok(crate::llm::client::build_default_template(
        &language,
        config.text_structuring,
    ))
}

#[tauri::command]
pub async fn preview_custom_prompt(template: String, enabled: bool) -> Result<String, AppError> {
    // Renders what the LLM would actually receive given the current toggle state.
    // - When `enabled` is false (or template is empty/whitespace): returns the built-in
    //   system prompt for the current language — exactly what the production path uses
    //   in default mode, with no safety footer appended.
    // - When `enabled` is true and template is non-empty: renders the user's template
    //   against current context (clipboard if referenced, vocabulary, user_tags, history,
    //   active_app=None) and appends the safety footer.
    let config = config::load()?;
    let history = history::load_history();
    let vocabulary = crate::vocabulary::load_vocabulary();

    if !enabled || template.trim().is_empty() {
        return Ok(crate::llm::client::build_system_prompt(
            &config.language,
            config.text_structuring,
            &vocabulary,
            &config.user_tags,
        ));
    }

    let clipboard = if template.contains("{{clipboard}}") {
        arboard::Clipboard::new().and_then(|mut cb| cb.get_text()).ok()
    } else {
        None
    };

    let ctx = crate::llm::template::TemplateContext {
        clipboard: clipboard.as_deref(),
        vocabulary: &vocabulary,
        user_tags: &config.user_tags,
        active_app: None,
        language: &config.language,
        history: &history,
    };

    let body = crate::llm::template::render_template(&template, &ctx);
    Ok(format!("{body}\n\n{}", crate::llm::client::safety_footer(&config.language)))
}
