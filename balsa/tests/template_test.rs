use balsa::{AsParameters, Balsa, BalsaParameters, BalsaTemplate};

struct TemplateParams {
    document_title: String,
    header_text: String,
}

impl AsParameters for TemplateParams {
    fn as_parameters(&self) -> balsa::BalsaParameters {
        BalsaParameters::new()
            .string("documentTitle", self.document_title.clone())
            .string("headerText", self.header_text.clone())
    }
}

#[test]
fn template_test() {
    let test_template = r#"
    <html>
        <head>
            <title>{{ documentTitle : string }}</title>
        </head>
        <body>
            <h1>{{ headerText : string }}</h1>
        </body>
    </html>
    "#;

    let expected_output = r#"
    <html>
        <head>
            <title>Title!!</title>
        </head>
        <body>
            <h1>Hello world :)</h1>
        </body>
    </html>
    "#;

    let template_builder = Balsa::from_string(test_template.to_string());
    let template = template_builder
        .build_struct::<TemplateParams>()
        .expect("Template should successfully compile");

    let input = TemplateParams {
        document_title: "Title!!".to_string(),
        header_text: "Hello world :)".to_string(),
    };

    let output = template
        .render_html_string(&input)
        .expect("Template should successfully render");

    assert_eq!(output, expected_output);
}
