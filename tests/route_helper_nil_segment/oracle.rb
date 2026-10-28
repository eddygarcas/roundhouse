# actionpack 8.1.4: what tests/route_helper_nil_segment.rs checks.
#
#     ruby tests/route_helper_nil_segment/oracle.rb
require "action_controller"
routes = ActionDispatch::Routing::RouteSet.new
routes.draw do
  resources :articles do
    resources :comments, only: [:create, :destroy]
  end
end
RouteHelpers = Class.new { include routes.url_helpers }.new

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
