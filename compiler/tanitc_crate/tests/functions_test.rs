use tanitc_analyzer::Analyzer;
use tanitc_codegen::c_generator::CodeGenStream;
use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_serializer::xml_writer::XmlWriter;

use pretty_assertions::assert_str_eq;

#[test]
fn function_def_test() {
    const SRC_TEXT: &str = "\nsafe func sum(a: f32, b: f32): f32 {\
                            \n    return a + b\
                            \n}\
                            \nunsafe func main() {\
                            \n    var ret: f32 = sum(a, b)\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let node = parser.parse_global_block().unwrap();

    {
        const EXPECTED: &str = "\n<function-definition name=\"sum\">\
                                \n    <attributes safety=\"safe\"/>\
                                \n    <return-type>\
                                \n        <type style=\"primitive\" name=\"f32\"/>\
                                \n    </return-type>\
                                \n    <parameters>\
                                \n        <variable-definition name=\"a\" is-global=\"false\" is-mutable=\"true\">\
                                \n            <type style=\"primitive\" name=\"f32\"/>\
                                \n        </variable-definition>\
                                \n        <variable-definition name=\"b\" is-global=\"false\" is-mutable=\"true\">\
                                \n            <type style=\"primitive\" name=\"f32\"/>\
                                \n        </variable-definition>\
                                \n    </parameters>\
                                \n    <return-statement>\
                                \n        <operation style=\"binary\" operation=\"+\">\
                                \n            <identifier name=\"a\"/>\
                                \n            <identifier name=\"b\"/>\
                                \n        </operation>\
                                \n    </return-statement>\
                                \n</function-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <attributes safety=\"unsafe\"/>\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"ret\" is-global=\"false\" is-mutable=\"false\">\
                                \n            <type style=\"primitive\" name=\"f32\"/>\
                                \n        </variable-definition>\
                                \n        <call-statement name=\"sum\">\
                                \n            <parameters>\
                                \n                <parameter index=\"0\">\
                                \n                    <identifier name=\"a\"/>\
                                \n                </parameter>\
                                \n                <parameter index=\"1\">\
                                \n                    <identifier name=\"b\"/>\
                                \n                </parameter>\
                                \n            </parameters>\
                                \n        </call-statement>\
                                \n    </operation>\
                                \n</function-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        node.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_str_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "float sum(float a, float b);\
                                     \nvoid main();\n";
        const SOURCE_EXPECTED: &str = "float sum(float a, float b){\
                                     \nreturn a + b;\
                                     \n}\
                                     \nvoid main(){\
                                     \nfloat const ret = sum(a, b);\
                                     \n}\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(HEADER_EXPECTED, header_res);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(SOURCE_EXPECTED, source_res);
    }
}

#[test]
fn functions_test() {
    const SRC_TEXT: &str = "\nfunc f(a: i32, b: i32): f32 {\
                            \n    return a + b\
                            \n}\
                            \n\
                            \nfunc main() {\
                            \n   var param = 34\
                            \n   var res = f(56, b: param)\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    let mut analyzer = Analyzer::new();
    {
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:#?}", analyzer.get_errors());
        }
    }

    {
        const EXPECTED: &str = "\n<function-definition name=\"f\">\
                                \n    <return-type>\
                                \n        <type style=\"primitive\" name=\"f32\"/>\
                                \n    </return-type>\
                                \n    <parameters>\
                                \n        <variable-definition name=\"a\" is-global=\"false\" is-mutable=\"true\">\
                                \n            <type style=\"primitive\" name=\"i32\"/>\
                                \n        </variable-definition>\
                                \n        <variable-definition name=\"b\" is-global=\"false\" is-mutable=\"true\">\
                                \n            <type style=\"primitive\" name=\"i32\"/>\
                                \n        </variable-definition>\
                                \n    </parameters>\
                                \n    <return-statement>\
                                \n        <operation style=\"binary\" operation=\"+\">\
                                \n            <identifier name=\"a\"/>\
                                \n            <identifier name=\"b\"/>\
                                \n        </operation>\
                                \n    </return-statement>\
                                \n</function-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"param\" is-global=\"false\" is-mutable=\"false\">\
                                \n            <type style=\"primitive\" name=\"i32\"/>\
                                \n        </variable-definition>\
                                \n        <literal style=\"integer-number\" value=\"34\"/>\
                                \n    </operation>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"res\" is-global=\"false\" is-mutable=\"false\">\
                                \n            <type style=\"primitive\" name=\"f32\"/>\
                                \n        </variable-definition>\
                                \n        <call-statement name=\"f\">\
                                \n            <parameters>\
                                \n                <parameter index=\"0\">\
                                \n                    <literal style=\"integer-number\" value=\"56\"/>\
                                \n                </parameter>\
                                \n                <parameter index=\"1\">\
                                \n                    <identifier name=\"param\"/>\
                                \n                </parameter>\
                                \n            </parameters>\
                                \n        </call-statement>\
                                \n    </operation>\
                                \n</function-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        program.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_str_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "float f(signed int a, signed int b);\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "float f(signed int a, signed int b){\
                                        \nreturn a + b;\
                                      \n}\
                                      \nvoid main(){\
                                        \nsigned int const param = 34;\
                                        \nfloat const res = f(56, param);\
                                      \n}\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let mut res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(HEADER_EXPECTED, res);

        res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(SOURCE_EXPECTED, res);
    }
}

#[test]
fn function_in_module_work_test() {
    const SRC_TEXT: &str = "\nmodule color {\
                            \n    enum Color {\
                            \n        Red\
                            \n        Green\
                            \n        Blue\
                            \n    }\
                            \n    func get_green(): Color {\
                            \n        var ret = Color::Green\
                            \n        return ret\
                            \n    }\
                            \n}\
                            \nfunc main() {\
                            \n    var green = color::get_green()\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }
    }

    {
        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:?}", analyzer.get_errors());
        }
    }

    {
        const EXPECTED: &str = "\n<module-definition name=\"color\">\
                                \n    <enum-definition name=\"Color\">\
                                \n        <field name=\"Red\" value=\"0\"/>\
                                \n        <field name=\"Green\" value=\"1\"/>\
                                \n        <field name=\"Blue\" value=\"2\"/>\
                                \n    </enum-definition>\
                                \n    <function-definition name=\"get_green\">\
                                \n        <return-type>\
                                \n            <type style=\"named\" name=\"Color\"/>\
                                \n        </return-type>\
                                \n        <operation style=\"binary\" operation=\"=\">\
                                \n            <variable-definition name=\"ret\" is-global=\"false\" is-mutable=\"false\">\
                                \n                <type style=\"named\" name=\"Color\"/>\
                                \n            </variable-definition>\
                                \n            <operation>\
                                \n                <literal style=\"integer-number\" value=\"1\"/>\
                                \n            </operation>\
                                \n        </operation>\
                                \n        <return-statement>\
                                \n            <identifier name=\"ret\"/>\
                                \n        </return-statement>\
                                \n    </function-definition>\
                                \n</module-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"green\" is-global=\"false\" is-mutable=\"false\">\
                                \n            <type style=\"tuple\"/>\
                                \n        </variable-definition>\
                                \n        <operation style=\"access\">\
                                \n            <identifier name=\"color\"/>\
                                \n            <call-statement name=\"get_green\"/>\
                                \n        </operation>\
                                \n    </operation>\
                                \n</function-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        program.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_str_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n    Red = 0,\
                                     \n    Green = 1,\
                                     \n    Blue = 2,\
                                     \n} Color;\
                                     \nColor get_green();\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "Color get_green(){\
                                     \nColor const ret = 1;\
                                     \nreturn ret;\
                                     \n}\
                                     \nvoid main(){\
                                     \nvoid const green = color.get_green();\
                                     \n}\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let mut res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(HEADER_EXPECTED, res);

        res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(SOURCE_EXPECTED, res);
    }
}

#[test]
fn incorrect_call_test() {
    const SRC_TEXT: &str = "\nfunc f(a: i32, b: i32): f32 {\
                            \n    return a + b\
                            \n}\
                            \n\
                            \nfunc main() {\
                            \n   var pi = 3.14\
                            \n   var res = f(5.6, b: pi)\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    let mut analyzer = Analyzer::new();
    {
        const EXPECTED_1: &str = "Semantic error: Mismatched types. In function \"f\" call: positional parameter \"0\" has type \"f32\" but expected \"i32\"";
        const EXPECTED_2: &str = "Semantic error: Mismatched types. In function \"f\" call: notified parameter \"b\" has type \"f32\" but expected \"i32\"";

        program.accept_mut(&mut analyzer).unwrap();
        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 2);
        assert_str_eq!(errors[0].text, EXPECTED_1);
        assert_str_eq!(errors[1].text, EXPECTED_2);
    }
}

#[test]
fn incorrect_notified_call_test() {
    const SRC_TEXT: &str = "\nfunc f(a: i32, b: i32): f32 {\
                            \n    return a + b\
                            \n}\
                            \n\
                            \nfunc main() {\
                            \n   var res = f(a: 44, 56)\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    let mut analyzer = Analyzer::new();
    {
        const EXPECTED: &str = "Semantic error: In function \"f\" call: positional parameter \"1\" must be passed before notified";

        program.accept_mut(&mut analyzer).unwrap();
        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 1);
        assert_str_eq!(errors[0].text, EXPECTED);
    }
}

#[test]
fn incorrect_module_func_call_test() {
    const SRC_TEXT: &str = "\nmodule math {\
                            \n    func f(a: i32, b: i32): f32 {\
                            \n        return a + b\
                            \n    }\
                            \n}\
                            \n\
                            \nfunc main() {\
                            \n   var pi = 3.14\
                            \n   var res = math::f(5.6, b: pi)\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    let mut analyzer = Analyzer::new();
    {
        const EXPECTED_1: &str = "Semantic error: Mismatched types. In function \"f\" call: positional parameter \"0\" has type \"f32\" but expected \"i32\"";
        const EXPECTED_2: &str = "Semantic error: Mismatched types. In function \"f\" call: notified parameter \"b\" has type \"f32\" but expected \"i32\"";

        program.accept_mut(&mut analyzer).unwrap();
        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 2);
        assert_str_eq!(errors[0].text, EXPECTED_1);
        assert_str_eq!(errors[1].text, EXPECTED_2);
    }
}
