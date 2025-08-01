use tanitc_analyzer::Analyzer;
use tanitc_codegen::c_generator::CodeGenStream;
use tanitc_parser::Parser;

use pretty_assertions::assert_str_eq;

#[test]
fn function_def_test() {
    const SRC_TEXT: &str = "\nsafe pub func sum(mut a: f32, b: f32): f32 {\
                            \n    return a + b\
                            \n}\
                            \nunsafe func main() {\
                            \n    var ret: f32 = sum(a, b)\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

    let node = parser.parse_global_block().unwrap();

    {
        const HEADER_EXPECTED: &str = "float sum(float a, float const b);\
                                     \nvoid main();\n";
        const SOURCE_EXPECTED: &str = "float sum(float a, float const b)\
                                     \n{\
                                     \n    return a + b;\
                                     \n}\
                                     \nvoid main()\
                                     \n{\
                                     \n    float const ret = sum(a, b);\
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
                            \nfunc void_func() {\
                            \n}\
                            \n\
                            \nfunc main() {\
                            \n   var param = 34\
                            \n   var res = f(56, b: param)\
                            \n   void_func()
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

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
        const HEADER_EXPECTED: &str = "float f(signed int const a, signed int const b);\
                                     \nvoid void_func();\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "float f(signed int const a, signed int const b)\
                                     \n{\
                                     \n    return a + b;\
                                     \n}\
                                     \nvoid void_func() { }\
                                     \nvoid main()\
                                     \n{\
                                     \n    signed int const param = 34;\
                                     \n    float const res = f(56, param);\
                                     \n    void_func();\
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

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

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
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n    Red = 0,\
                                     \n    Green = 1,\
                                     \n    Blue = 2,\
                                     \n} Color;\
                                     \nColor get_green();\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "Color get_green()\
                                     \n{\
                                     \n    Color const ret = 1;\
                                     \n    return ret;\
                                     \n}\
                                     \nvoid main()\
                                     \n{\
                                     \n    void const green = color.get_green();\
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

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

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

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

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

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

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
