extern crate jets;

use jets::core::*;

#[test]
#[ignore]
fn test_index() {
    let mut writer = IndexWriter::new("/Users/jeffsky/jets");
    for i in 0..10 {
        let mut doc = Document::new(0);
        writer.add_document(doc);
    }
    writer.close();
}
