#[macro_use]
extern crate log;
extern crate jets;

use jets::analysis::{JiebaTokenizer, Tokenizer};
use jets::core::*;
use jets::search::*;
use std::time::Instant;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

static DIR: &str = "/tmp/jets/TEST";
static AMOUNT: u64 = 10000;

#[test]
fn test_index_write() {
    init();
    let mut writer = IndexWriter::open(DIR, JiebaTokenizer::default()).unwrap();
    for i in 0..AMOUNT {
        writer.push(mock_doc(i)).unwrap();
    }
    writer.flush().unwrap();
}

#[test]
fn test_index_read() {
    init();
    let now = Instant::now();
    let reader = IndexReader::open(DIR).unwrap();
    info!(">>>> open index: cost={}ms", now.elapsed().as_millis());
    for i in 0..3 {
        let doc = reader.document(i).unwrap();
        println!("next #{}: {:?}", i, doc);
    }
    let founds = reader
        .find("nickname", DocValue::String("bar_8888".to_string()))
        .unwrap();
    for id in founds.iter() {
        let result = reader.document(*id).unwrap();
        info!("####### found: {:?}", result);
    }
}

fn mock_doc(id: u64) -> Document {
    let f1 = Field::new(
        "name".to_string(),
        DocValue::String(format!("foo_{}", id)),
        0,
    );
    let f2 = Field::new(
        "nickname".to_string(),
        DocValue::String(format!("bar_{}", id)),
        0,
    );
    Document::builder(id).put(f1).put(f2).build()
}

#[test]
fn test_text_index_write() {
    init();
    let inputs = vec![
        "我爱北京天安门",
        "上海是我们的家",
        "我们中出了个叛徒",
        "北京有长城",
    ];
    let path = "/tmp/jets/TEST_TEXT";
    let mut writer = IndexWriter::open(path, JiebaTokenizer::default()).unwrap();
    for i in 0..inputs.len() {
        let doc = Document::builder(i as u64)
            .put(Field::new(
                "content".to_string(),
                DocValue::Text(inputs[i].to_string()),
                0,
            ))
            .build();
        writer.push(doc).unwrap();
    }
    writer.flush().unwrap();
}

#[test]
fn test_fulltext_index_searcher() {
    init();
    let path = "/tmp/jets/TEST_TEXT";
    let reader = IndexReader::open(path).unwrap();
    let searcher = IndexSearcher::from(reader);

    let submit = |q: Query| {
        let found = searcher.search(&q).documents().unwrap();
        for it in found {
            info!("search result: {:?}", it);
        }
        info!("---------------------------------");
    };

    submit(Query::from(Condition::Term(
        "content".to_string(),
        "长城".to_string(),
    )));

    submit(Query::from(Condition::Group(
        GroupOp::OR,
        vec![
            Condition::Term("content".to_string(), "上海".to_string()),
            Condition::Term("content".to_string(), "北京".to_string()),
        ],
    )));

    submit(Query::from(Condition::Group(
        GroupOp::AND,
        vec![
            Condition::Term("content".to_string(), "长城".to_string()),
            Condition::Term("content".to_string(), "北京".to_string()),
        ],
    )));
}
