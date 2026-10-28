//! A path helper given a nil or empty required segment raises
//! `ActionController::UrlGenerationError` with Rails' message, as
//! actionpack 8.1.4 does, instead of building "/articles//comments"
//! (roundhouse#1475). Its oracle, on the same calls:
//!
//!     ruby tests/route_helper_nil_segment/oracle.rb

#[path = "support/emit_and_run.rs"]
#[allow(dead_code)]
mod emit_and_run;

const SCRIPT: &str = r##"
def check(label, got, want)
  raise "#{label}: #{got.inspect} (want #{want.inspect})" unless got == want
end
def error_of
  yield
  "no error"
rescue ActionController::UrlGenerationError => e
  e.message
end
check("a nil segment", error_of { RouteHelpers.article_comments_path(nil) },
  'No route matches {action: "create", article_id: nil, controller: "comments"}, missing required keys: [:article_id]')
check("an empty segment", error_of { RouteHelpers.article_comments_path("") },
  'No route matches {action: "create", article_id: "", controller: "comments"}, missing required keys: [:article_id]')
check("one of two segments", error_of { RouteHelpers.article_comment_path(nil, 5) },
  'No route matches {action: "destroy", article_id: nil, controller: "comments", id: "5"}, missing required keys: [:article_id]')
check("both segments", error_of { RouteHelpers.article_comment_path(nil, nil) },
  'No route matches {action: "destroy", article_id: nil, controller: "comments", id: nil}, missing required keys: [:article_id, :id]')
check("a filled segment", RouteHelpers.article_comments_path(1), "/articles/1/comments")
puts "nil segment passed"
"##;

#[test]
fn a_nil_required_segment_raises_on_ruby() {
    let run = emit_and_run::real_blog().run_ruby(SCRIPT);
    run.assert_passes();
    assert!(run.stdout.contains("nil segment passed"), "{}", run.stdout);
}

#[test]
#[ignore = "requires the Spinel toolchain"]
fn a_nil_required_segment_raises_on_spinel() {
    let run = emit_and_run::real_blog().run_spinel_with_db(SCRIPT);
    run.assert_passes();
    assert!(run.stdout.contains("nil segment passed"), "{}", run.stdout);
}
