use lancea_provider_apps::AppsProvider;

#[test]
fn scan_and_find_something() {
    println!("running...");
    let p = AppsProvider::new().expect("scan");
    let results = p.search("firefox");
    println!("scanned apps: {}", p.search("firefox").len());

    assert_eq!(results.first().unwrap().title, "Firefox");
}

#[test]
fn search_is_case_insensitive_and_fuzzy() {
    let p = AppsProvider::new().expect("scan");

    let results = p.search("visual");
    assert_eq!(results.first().unwrap().title, "Visual Studio Code");
}
