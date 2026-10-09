//! rubys/roundhouse#729: `ActiveStorage::AttachedMany` has no `find(id)`
//! (Rails forwards it to `attachments` via `delegate_missing_to`), and
//! since `AttachedMany#each` (e2e9ff1a, #717) Spinel binds `find(id)` to
//! the blockless `Enumerable#find`, so `.blob` / `.purge` on the result
//! are refused at `spin build` time (an untyped Enumerator). `Attachment
//! #purge` was also missing on the per-row `ManyAttachment` value.
//! (Kept out of tests/emit_and_run.rs so concurrent appends there do not
//! conflict; same harness.)

#[path = "support/emit_and_run.rs"]
mod emit_and_run;

const ACTIVE_STORAGE_TABLES: &str = r#"  create_table "active_storage_blobs", force: :cascade do |t|
    t.string "key", null: false
    t.string "filename", null: false
    t.string "content_type"
    t.text "metadata"
    t.string "service_name", null: false
    t.bigint "byte_size", null: false
    t.string "checksum"
    t.datetime "created_at", null: false
  end

  create_table "active_storage_attachments", force: :cascade do |t|
    t.string "name", null: false
    t.string "record_type", null: false
    t.bigint "record_id", null: false
    t.bigint "blob_id", null: false
    t.datetime "created_at", null: false
  end

  create_table "active_storage_variant_records", force: :cascade do |t|
    t.bigint "blob_id", null: false
    t.string "variation_digest", null: false
  end

  add_foreign_key "comments", "articles"
"#;

fn overlay() -> emit_and_run::Overlay {
    emit_and_run::real_blog()
        .edit(
            "db/schema.rb",
            "  add_foreign_key \"comments\", \"articles\"\n",
            ACTIVE_STORAGE_TABLES,
        )
        .edit(
            "app/models/article.rb",
            "class Article < ApplicationRecord\n",
            "class Article < ApplicationRecord\n  has_many_attached :photos\n",
        )
}

/// The exact shape issue #729 repros: `widget.photos.find(photo_id)`
/// (a String, as `params[:photo_id]` always is) must return the
/// matching join row — not an Enumerable#find Enumerator — so `.blob`
/// and `.purge` run on it, and an id naming no row raises
/// ActiveRecord::RecordNotFound the way Rails does.
const FIND_AND_PURGE_SCRIPT: &str = r##"
article = Article.create!(title: "Photos", body: "A body long enough to pass.")
blob1 = ActiveStorage::Blob.create_and_upload!("one", "one.txt", "text/plain")
blob2 = ActiveStorage::Blob.create_and_upload!("two", "two.txt", "text/plain")
article.photos.attach_blob(blob1)
article.photos.attach_blob(blob2)
first, second = article.photos.attachments
raise "fixture needs two rows: #{article.photos.attachments.length}" unless article.photos.attachments.length == 2

photo = article.photos.find(second.id.to_s)
raise "find returned the wrong row" unless photo.blob.filename.to_s == "two.txt"

photo.purge
remaining = article.photos.attachments
raise "purge left #{remaining.length} rows" unless remaining.length == 1
raise "purge kept the wrong row" unless remaining[0].id == first.id
raise "purge left the blob behind" unless ActiveStorage::Blob.count == 1

begin
  article.photos.find(second.id)
  raise "find did not raise for a purged id"
rescue ActiveRecord::RecordNotFound
end

begin
  article.photos.find("not-an-id")
  raise "find did not raise for an unknown id"
rescue ActiveRecord::RecordNotFound
end

puts "attached_many find/purge ok"
"##;

#[test]
fn attached_many_find_returns_the_row_and_purge_removes_it() {
    overlay().run_ruby(FIND_AND_PURGE_SCRIPT).assert_passes();
}

/// Same shape, compiled by Spinel: before the fix `spin build` refuses
/// `.blob` / `.purge` (the issue's 2 refusals) because `find(id)` binds
/// to the blockless `Enumerable#find` once `AttachedMany` has an `each`.
/// `run_spinel`'s consumer boots the emitted libraries directly, so the
/// database needs the same manual setup other `_on_spinel` tests give it.
#[test]
#[ignore = "requires the Spinel toolchain"]
fn attached_many_find_and_purge_run_on_spinel() {
    let script = format!(
        "Db.configure(\":memory:\")\nSchema.statements.each {{ |sql| Db.exec(sql) }}\nActiveRecord.adapter = SqliteAdapter\n{FIND_AND_PURGE_SCRIPT}"
    );
    overlay().run_spinel(&script).assert_passes();
}
