use swc_core::{
    common::{FileName, FilePathMapping, SourceMap, Span},
    ecma::ast::Expr,
    plugin::errors::HANDLER,
};
use swc_ecma_parser::{parse_file_as_expr, EsConfig, PResult, Syntax};

pub fn json_path_from_key(value: String, scopes: &[String]) -> (String, String) {
    if let Some(keys) = value.split_once(':') {
        (keys.0.to_string(), keys.1.to_string())
    } else {
        (scopes.get(0).unwrap_or(&"".to_string()).clone(), value)
    }
}

pub fn parse_code(code: String) -> PResult<Box<Expr>> {
    let cm = SourceMap::new(FilePathMapping::empty());
    let fm = cm.new_source_file(FileName::Custom("".into()), code);
    parse_file_as_expr(
        &fm,
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        Default::default(),
        None,
        &mut vec![],
    )
}

pub fn emit_error(message: &str, span: Span) {
    if cfg!(test) {
        panic!("{}", message);
    }
    HANDLER.with(|handler| {
        handler.struct_span_err(span, message).emit();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_path_from_key() {
        let scopes = vec!["module".to_string(), "data".to_string()];
        let result = json_path_from_key("variable:name".to_string(), &scopes);
        assert_eq!(result, ("variable".to_string(), "name".to_string()));
    }

    #[test]
    fn test_parse_code() {
        let code = "<>foo</>".to_string();
        let result = parse_code(code);
        assert!(result.is_ok());
    }

    #[test]
    #[should_panic(expected = "some error")]
    fn test_emit_error() {
        let cm = SourceMap::new(FilePathMapping::empty());
        let fm = cm.new_source_file(FileName::Custom("".into()), "".into());
        let span = Span::new(fm.start_pos, fm.end_pos, Default::default());
        emit_error("some error", span);
    }
}
