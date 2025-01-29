fn main() {
    embed_resource::compile("pikaconfig-manifest.rc", embed_resource::NONE)
        .manifest_optional()
        .unwrap();
    lalrpop::process_src().unwrap();
}
