use tanitc_serializer::XmlWriter;

#[test]
fn writer_test() {
    const EXPECTED: &str = "\n<empty-tag/>\
                            \n<tag-with-param-without-children int=\"200\" float=\"100.5\" str=\"some_string\"/>\
                            \n<tag-with-child-without-params>\
                            \n    <empty-child-tag/>\
                            \n    <not-empty-child-tag key=\"value\"/>\
                            \n</tag-with-child-without-params>";

    let mut buffer = Vec::<u8>::new();
    let mut writer = XmlWriter::new(&mut buffer).unwrap();

    writer.begin_tag("empty-tag").unwrap();
    writer.end_tag().unwrap();

    writer.begin_tag("tag-with-param-without-children").unwrap();
    writer.put_param("int", 200).unwrap();
    writer.put_param("float", 100.500).unwrap();
    writer.put_param("str", "some_string").unwrap();
    writer.end_tag().unwrap();

    writer.begin_tag("tag-with-child-without-params").unwrap();
    {
        writer.begin_tag("empty-child-tag").unwrap();
        writer.end_tag().unwrap();

        writer.begin_tag("not-empty-child-tag").unwrap();
        writer.put_param("key", "value").unwrap();
        writer.end_tag().unwrap();
    }
    writer.end_tag().unwrap();

    let res = String::from_utf8(buffer).unwrap();
    assert_eq!(EXPECTED, res);
}
