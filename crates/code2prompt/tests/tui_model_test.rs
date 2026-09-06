use code2prompt::model::{Cmd, Message, Model};
use code2prompt_core::{
    configuration::Code2PromptConfig, session::Code2PromptSession, template::OutputFormat,
};

fn model_with_format(output_format: OutputFormat) -> Model {
    let config = Code2PromptConfig::builder()
        .path(std::env::temp_dir())
        .output_format(output_format)
        .build()
        .unwrap();
    Model::new(Code2PromptSession::new(config))
}

#[test]
fn tui_uses_xml_default_for_an_xml_session() {
    let model = model_with_format(OutputFormat::Xml);

    assert!(model.template.uses_automatic_template);
    assert!(model.template.get_template_content().contains("<files>"));

    let (_, cmd) = model.update(Message::RunAnalysis);
    match cmd {
        Cmd::RunAnalysis {
            template_content, ..
        } => assert!(template_content.is_empty()),
        other => panic!("expected analysis command, got {other:?}"),
    }
}

#[test]
fn space_cycles_output_format_and_updates_the_automatic_template() {
    let model = model_with_format(OutputFormat::Markdown);

    // Output Format is the fourth item in the flattened settings list.
    let (model, _) = model.update(Message::ToggleSetting(3));
    let (model, _) = model.update(Message::ToggleSetting(3));

    assert_eq!(model.session.config.output_format, OutputFormat::Xml);
    assert!(model.template.uses_automatic_template);
    assert!(model.template.get_template_content().contains("<files>"));
    assert!(!model.template.get_template_content().contains("```"));
}

#[test]
fn tui_preserves_an_explicit_template_from_the_session() {
    let config = Code2PromptConfig::builder()
        .path(std::env::temp_dir())
        .output_format(OutputFormat::Xml)
        .template_str("Custom: {{absolute_code_path}}".to_string())
        .template_name("custom".to_string())
        .build()
        .unwrap();
    let model = Model::new(Code2PromptSession::new(config));

    assert!(!model.template.uses_automatic_template);
    assert_eq!(
        model.template.get_template_content(),
        "Custom: {{absolute_code_path}}"
    );

    let (_, cmd) = model.update(Message::RunAnalysis);
    match cmd {
        Cmd::RunAnalysis {
            template_content, ..
        } => assert_eq!(template_content, "Custom: {{absolute_code_path}}"),
        other => panic!("expected analysis command, got {other:?}"),
    }
}
