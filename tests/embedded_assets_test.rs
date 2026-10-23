use ezsn_quiz::static_files::Asset;

#[test]
fn test_embedded_dist_assets() {
    // 1. Verify index.html is embedded
    let index_html = Asset::get("index.html");
    assert!(index_html.is_some(), "frontend/dist/index.html must be embedded");
    
    let file = index_html.unwrap();
    let index_str = std::str::from_utf8(&file.data).expect("index.html must be valid utf8");
    assert!(index_str.contains("<div id=\"app\"></div>"), "index.html must contain Vue mount point");

    // 2. Verify favicon is embedded
    assert!(Asset::get("favicon.ico").is_some(), "favicon.ico must be embedded");
    assert!(Asset::get("favicon.svg").is_some(), "favicon.svg must be embedded");

    // 3. Verify CSS & JS bundle chunks are embedded in assets/
    let mut found_js = false;
    let mut found_css = false;

    for file in Asset::iter() {
        if file.starts_with("assets/") && file.ends_with(".js") {
            found_js = true;
        }
        if file.starts_with("assets/") && file.ends_with(".css") {
            found_css = true;
        }
    }

    assert!(found_js, "Must embed at least one compiled JS bundle in assets/");
    assert!(found_css, "Must embed at least one compiled CSS bundle in assets/");
}
