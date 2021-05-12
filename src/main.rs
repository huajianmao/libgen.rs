use scraper::{Html, Selector};

struct Book {
    id: String,
    title: String,
    // url: String,
    md5: String,
    // authors: Vec<String>,
    // publisher: String,
    // year: i32,
    // pages: i32,
    // language: String,
    // size: String,
    extension: String,
    // mirrors: Vec<String>,
    files: Vec<String>,
}

struct Params {
    search_word: String,
    download_type: i32,      // 0, 1, 2, 3
    results_per_page: i32,   // 25, 50, 100
    view_results: String,    // "simple", "detailed"
    search_with_mask: i32,   // "0 - No, 1 - Yes"
    search_in_field: String, // "def", "title", "author", "series", "publisher", "year", "isbn", "language", "md5", "tags", "extension"
}

impl Params {
    fn to_query_params(&self) -> [(&str, String); 6] {
        return [
            ("req", self.search_word.to_string()),
            ("open", self.download_type.to_string()),
            ("res", self.results_per_page.to_string()),
            ("view", self.view_results.to_string()),
            ("phrase", self.search_with_mask.to_string()),
            ("column", self.search_in_field.to_string()),
        ];
    }
}

struct Libgen {
    client: reqwest::blocking::Client
}

impl Libgen {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new()
        }
    }

    fn search(&self, search_word: &str) -> Result<Vec<Book>, reqwest::Error> {
        let search_url = "http://libgen.rs/search.php";

        let table_selector = Selector::parse("table.c").unwrap();
        let tr_selector = Selector::parse("tr").unwrap();

        let params = Params {
            search_word: search_word.to_string(),
            download_type: 0,
            results_per_page: 25,
            view_results: "simple".to_string(),
            search_with_mask: 0,
            search_in_field: "def".to_string(),
        };
        let resp = self.client.get(search_url).query(&params.to_query_params()).send()?.text().unwrap();
        let document = Html::parse_document(&resp);
        let table = document.select(&table_selector).next().unwrap();
        let trs = table.select(&tr_selector).skip(1);
        let mut books = Vec::new();
        for tr in trs {
            let book = self.parse(&tr).unwrap();
            books.push(book);
        }
        Ok(books)
    }

    fn parse(&self, tr: &scraper::ElementRef) -> Result<Book, reqwest::Error> {
        let td_selector = Selector::parse("td").unwrap();
        let a_selector = Selector::parse("a").unwrap();

        let mut tds = tr.select(&td_selector);
        let td = tds.nth(2).unwrap();
        let a = td.select(&a_selector).last().unwrap();
        let token = "book/index.php?md5=";
        let extension = tds.nth(5).unwrap().inner_html().to_string();

        let id = a.value().attr("id").unwrap().trim().to_string();
        let url = a.value().attr("href").unwrap().trim().to_string();
        let md5 = (&url[token.chars().count()..]).to_string();
        let title = a.first_child().unwrap().value().as_text().unwrap().trim().to_string();

        let book = Book {id, title, md5, extension, files: Default::default()};

        Ok(book)
    }

    fn load_file_urls(&self, book: &mut Book) -> Result<(), reqwest::Error> {
        let mirror_url = "http://library.lol/main/";
        let book_url = format!("{}{}", mirror_url, book.md5);
        let download_selector = Selector::parse("#download").unwrap();
        let li_selector = Selector::parse("li").unwrap();
        let a_selector = Selector::parse("a").unwrap();

        let book_page = self.client.get(book_url).send()?.text().unwrap();
        let book_document = Html::parse_document(&book_page);
        let download = book_document.select(&download_selector).next().unwrap();
        let lis = download.select(&li_selector);
        for li in lis {
            let pdf = li.select(&a_selector).nth(0).unwrap().value().attr("href").unwrap().trim();
            book.files.push(pdf.to_string());
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let libgen = Libgen::new();
    let args: Vec<String> = std::env::args().collect();

    // println!("Please input the search word to find book: ");
    // let mut search_word = String::new();
    // let _ = std::io::stdin().read_line(&mut search_word);
    // search_word = search_word.trim().to_string();

    if args.len() <= 1 || args[1].len() <= 1 {
        println!("The search query length should be not less than 2 characters!");
        std::process::exit(-1);
    }
    let search_word= &args[1];

    let mut books = libgen.search(&search_word).unwrap();

    for (i, book) in books.iter().enumerate() {
        println!("[{}]\t{}\t{}[.{}]", i + 1, book.id, book.title, book.extension);
    }
    println!("Input the number to download: ");

    let mut line = String::new();
    let _ = std::io::stdin().read_line(&mut line);
    let number: usize = std::str::FromStr::from_str(&line.trim()).unwrap_or(0);

    println!("The input is {}", number);
    if number >= books.len() + 1 || number <= 0 {
        println!("The number should be larger than 0 and less than {}", books.len());
    } else {
        let book = &mut books[number - 1];
        println!("Going to download [{}] {}", number, book.title);
        let _ = libgen.load_file_urls(book);
        println!("{:#?}", book.files);
    }

    Ok(())
}
