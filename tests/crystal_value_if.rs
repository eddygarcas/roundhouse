//! A value-position `if`/`else` inside an index `[...]` or a call
//! argument must emit as a Crystal expression the parser accepts there.
//! The block form `if c\n a\nelse\n b\nend` inside `[...]` is a Crystal
//! syntax error ("expecting token ']', not 'if'"), which the runtime
//! router hits with `path_parts[glob >= 0 && i > glob ? i + extra : i]`.
use roundhouse::analyze::Analyzer;
use std::collections::HashMap;
use std::path::PathBuf;

fn method_body(source: &str, name: &str) -> String {
    let mut app = roundhouse::ingest::ingest_app_from_tree(HashMap::from([(
        PathBuf::from("app/lib/value_if_probe.rb"),
        source.as_bytes().to_vec(),
    )]))
    .unwrap();
    Analyzer::new(&app).analyze(&mut app);
    let class = app
        .library_classes
        .iter()
        .find(|c| c.name.0.as_str() == "ValueIfProbe")
        .unwrap();
    let body = &class
        .methods
        .iter()
        .find(|m| m.name.as_str() == name)
        .unwrap()
        .body;
    roundhouse::emit::crystal::emit_expr_for_runtime(body)
}

const SOURCE: &str = r#"class ValueIfProbe
  def pick(items, flag, i)
    items[flag ? i + 1 : i]
  end

  def pass(items, flag, i)
    items.fetch(flag ? i + 1 : i)
  end

  def nested(items, flag, other, i)
    items[flag ? (other ? i + 2 : i + 1) : i]
  end

  def assigned(flag, i)
    x = flag ? i + 1 : i
    x
  end
end
"#;

#[test]
fn ternary_in_index_emits_inline() {
    let out = method_body(SOURCE, "pick");
    assert!(!out.contains('\n'), "index must stay on one line:\n{out}");
    assert!(out.contains("items[(flag ? i + 1_i64 : i)]"), "got:\n{out}");
}

#[test]
fn ternary_in_call_argument_emits_inline() {
    let out = method_body(SOURCE, "pass");
    assert!(!out.contains('\n'), "argument must stay on one line:\n{out}");
    assert!(out.contains("items.fetch((flag ? i + 1_i64 : i))"), "got:\n{out}");
}

#[test]
fn multi_line_value_if_in_index_is_parenthesized() {
    let out = method_body(SOURCE, "nested");
    assert!(out.contains("items[(if flag"), "got:\n{out}");
    assert!(out.trim_end().ends_with("end)]"), "got:\n{out}");
}

#[test]
fn assignment_position_keeps_block_form() {
    let out = method_body(SOURCE, "assigned");
    assert!(out.contains("x = if flag\n"), "got:\n{out}");
}
