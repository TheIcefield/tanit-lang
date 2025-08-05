use tanitc_analyzer::Analyzer;
use tanitc_codegen::c_generator::CodeGenStream;
use tanitc_parser::Parser;

use pretty_assertions::assert_str_eq;

#[test]
fn struct_work_test() {
    const SRC_TEXT: &str = "\nstruct MyStruct\
                            \n{\
                            \n    pub f1: i32\
                            \n    f2: [f32: 2]\
                            \n}\
                            \nfunc main() {\
                            \n    var mut s = MyStruct { \
                            \n              f1: 1, f2: [2.0, 3.0]\
                            \n            }\
                            \n    s.f1 = 2\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Failed to create parser");

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
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n    signed int f1;\
                                     \n    float f2;\
                                     \n} MyStruct;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main()\
                                     \n{\
                                     \n    MyStruct s = (MyStruct)\
                                     \n    {\
                                     \n        .f1=1,\
                                     \n        .f2={ 2.0, 3.0 },\
                                     \n    };\
                                     \n    s.f1 = 2;\
                                     \n}\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        let source_res = String::from_utf8(source_buffer).unwrap();

        assert_str_eq!(HEADER_EXPECTED, header_res);
        assert_str_eq!(SOURCE_EXPECTED, source_res);
    }
}

#[test]
fn struct_in_module_work_test() {
    const SRC_TEXT: &str = "\nmodule math {\
                            \n    struct Vector2 {\
                            \n        x: f32\
                            \n        y: f32\
                            \n    }\
                            \n}\
                            \nfunc main() {\
                            \n    var mut vec = math::Vector2 { \
                            \n              x: 0.0, y: 2.0\
                            \n            }\
                            \n    vec.x = 2.0\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Failed to create parser");

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
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n    float x;\
                                     \n    float y;\
                                     \n} Vector2;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main()\
                                     \n{\
                                     \n    Vector2 vec = (Vector2)\
                                     \n    {\
                                     \n        .x=0.0,\
                                     \n        .y=2.0,\
                                     \n    };\
                                     \n    vec.x = 2.0;\
                                     \n}\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(HEADER_EXPECTED, header_res);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(SOURCE_EXPECTED, source_res);
    }
}

#[test]
fn incorrect_struct_work_test() {
    const SRC_TEXT: &str = "\nstruct MyStruct\
                            \n{\
                            \n    f1: i32\
                            \n    f2: f32\
                            \n}\
                            \nfunc main() {\
                            \n    var mut s = MyStruct { \
                            \n              f1: 1, f2: 2.0\
                            \n            }\
                            \n    s.f1 = 3.0\
                            \n    s.f2 = 2.0\
                            \n    s.f3 = 1.0\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Failed to create parser");

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }
    }

    {
        const EXPECTED_1: &str =
            "Semantic error: Cannot perform operation on objects with different types: i32 and f32";
        const EXPECTED_2: &str = "Semantic error: \"s\" doesn't have member named \"f3\"";

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 2);
        assert_str_eq!(errors[0].text, EXPECTED_1);
        assert_str_eq!(errors[1].text, EXPECTED_2);
    }
}

#[test]
fn internal_struct_work_test() {
    const SRC_TEXT: &str = "\nmodule math {\
                            \n    struct Unit {\
                            \n        value: f32\
                            \n    }\
                            \n    struct Point2 {\
                            \n        x: Unit\
                            \n        y: Unit\
                            \n    }\
                            \n}\
                            \nfunc main() {\
                            \n    var mut pnt = math::Point2 { \
                            \n                  x: Unit { value: 1.0 },\
                            \n                  y: Unit { value: 2.0 },\
                            \n            }\
                            \n    pnt.x.value = 2.0\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Failed to create parser");

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
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n    float value;\
                                     \n} Unit;\
                                     \ntypedef struct {\
                                     \n    Unit x;\
                                     \n    Unit y;\
                                     \n} Point2;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main()\
                                     \n{\
                                     \n    Point2 pnt = (Point2)\
                                     \n    {\
                                     \n        .x=(Unit)\
                                     \n        {\
                                     \n            .value=1.0,\
                                     \n        },\
                                     \n        .y=(Unit)\
                                     \n        {\
                                     \n            .value=2.0,\
                                     \n        },\
                                     \n    };\
                                     \n    pnt.x.value = 2.0;\
                                     \n}\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(HEADER_EXPECTED, header_res);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(SOURCE_EXPECTED, source_res);
    }
}
