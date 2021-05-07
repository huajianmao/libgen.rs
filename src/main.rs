use scraper::{Html, Selector};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "http://libgen.rs/search.php";
    let search_word = "Biological Sequence Analysis";
    let download_type = 0; // 0, 1, 2, 3
    let results_per_page = 20; // 20, 50, 100
    let view_results = "simple"; // "simple", "detailed"
    let search_with_mask = 0; // "0 - No, 1 - Yes"
    let search_in_field = "def"; // "def", "title", "author", "series", "publisher", "year", "isbn", "language", "md5", "tags", "extension"
    
    let client = reqwest::blocking::Client::new();
    let resp = client.get(url).query(&[
        ("req", search_word),
        ("open", &download_type.to_string()),
        ("res", &results_per_page.to_string()),
        ("view", view_results),
        ("phrase", &search_with_mask.to_string()),
        ("column", search_in_field)
    ]).send()?.text().unwrap();

    let table_selector = Selector::parse("table.c").unwrap();
    let tr_selector = Selector::parse("tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();

    let document = Html::parse_document(&resp);
    let table = document.select(&table_selector).next().unwrap();
    let trs = table.select(&tr_selector).skip(1);
    for tr in trs {
        let mut tds = tr.select(&td_selector);
        let td = tds.nth(2);
        println!("{:#?}", td.unwrap().inner_html());
        // for td in tds {
        //     println!("{:#?}", td.inner_html());
        // }
    }

    Ok(())
}