use jieba_rs::Jieba;

#[test]
fn test_jieba_cut() {
    let inputs = vec![
        "我爱北京天安门",
        "上海是我们的家",
        "我们中出了个叛徒",
        "北京有长城",
        "在伦敦奥运会上将可能有一位沙特阿拉伯的女子"
        
    ];
    let jieba = Jieba::new();
    for input in inputs {
        let words = jieba.cut_for_search(input, false);
        println!("words: {:?}", words);
    }
}

#[test]
fn test_dup() {
    let mut ids: Vec<_> = vec![4u64, 2, 2, 5, 8, 7, 1, 66, 7, 1];
    ids.sort();
    ids = Some(ids[0])
        .into_iter()
        .chain(ids.windows(2).filter(|w| w[0] != w[1]).map(|w| w[1]))
        .collect();
    println!("ids: {:?}", ids);
}
