#[macro_use]
extern crate log;
extern crate clap;
extern crate jets;

use clap::{App, Arg};
use jets::analysis::JiebaTokenizer;
use jets::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::result::Result;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Debug)]
struct FilmInfo {
    title: String,
    year: String,
    star: f32,
    director: String,
    film_page: String,
}

fn init() {
    env_logger::builder().format_timestamp_millis().init();
}

// Download(TOKEN: auw6)
// https://link.zhihu.com/?target=https%3A//pan.baidu.com/s/12IiX4p_fLg8CyidAjl8_Zw

// BUILD INDEX:
// cargo run --example douban -- --input /Users/jeffsky/Downloads/Film.json --output /tmp/jets/douban --search 恶魔

// SEARCH INDEX:
// cargo run --example douban -- --output /tmp/jets/douban --search 恶魔
fn main() -> Result<(), Box<dyn Error>> {
    init();

    let cli = App::new("douban")
        .version("0.1.0")
        .author("Jeffsky <jjeffcaii@outlook.com>")
        .about("an douban film index demo.")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .required(true)
                .takes_value(true)
                .help("input film file."),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .required(true)
                .takes_value(true)
                .help("index ouput dir."),
        )
        .arg(
            Arg::with_name("search")
                .short("s")
                .long("search")
                .required(false)
                .takes_value(true)
                .help("search film."),
        )
        .get_matches();

    let input = cli.value_of("input").unwrap();
    let output = cli.value_of("output").unwrap();
    let search = cli.value_of("search");

    match search {
        None => {
            let f = File::open(input)?;
            let mut reader = BufReader::new(f);
            let mut line = String::new();
            let mut writer = IndexWriter::open(output, JiebaTokenizer::default())?;
            while let Ok(read) = reader.read_line(&mut line) {
                if read < 1 {
                    break;
                }
                if let Ok(film) = serde_json::from_str::<FilmInfo>(&line) {
                    debug!("**** read: {:?}", film);
                    let doc = Document::builder()
                        .put("title", DocValue::Text(film.title), FLAG_TOKENIZED)
                        .put("year", DocValue::Text(film.year), 0)
                        .build();
                    writer.push(doc)?;
                }
                line.clear();
            }
            writer.flush()?;
        }
        Some(word) => {
            let reader = IndexReader::open(output)?;
            let searcher = IndexSearcher::from(reader);
            let q = Query::from(Condition::Term("title".to_string(), word.to_string()));
            let mut now = Instant::now();
            let tops = searcher.search(&q);
            let cost1 = now.elapsed();
            now = Instant::now();
            let result = tops.documents();
            let cost2 = now.elapsed();
            let mut amount = 0usize;
            if let Some(docs) = result {
                for doc in docs.iter() {
                    amount += 1;
                    info!("found: {}", doc.get("title").unwrap());
                }
            }
            info!("-------------------------------------");
            let cost = Duration::from_nanos((cost1.as_nanos() + cost2.as_nanos()) as u64);
            info!(
                "amount={}, cost={}ms ({}ns: index={}, docs={})",
                amount,
                cost.as_millis(),
                cost.as_nanos(),
                cost1.as_nanos(),
                cost2.as_nanos(),
            );
        }
    };
    Ok(())
}
