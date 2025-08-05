use tanitc_analyzer::Analyzer;
use tanitc_codegen::c_generator::CodeGenStream;
use tanitc_parser::Parser;

use pretty_assertions::assert_str_eq;

#[test]
fn struct_with_methods_test() {
    const SRC_TEXT: &str = "\nstruct MyStruct\
                            \n{\
                            \n    f1: i32\
                            \n    f2: f32\
                            \n}\
                            \nimpl MyStruct\
                            \n{\
                            \n    func new(): MyStruct {\
                            \n        return MyStruct {\
                            \n                 f1: 0, f2: 0.0\
                            \n               }\
                            \n    }
                            \n}\
                            \nfunc main() {\
                            \n    var s = MyStruct { \
                            \n              f1: 1, f2: 2.0\
                            \n            }\
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
                                     \nMyStruct MyStruct__new();\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "MyStruct MyStruct__new()\
                                     \n{\
                                     \n    return (MyStruct)\
                                     \n    {\
                                     \n        .f1=0,\
                                     \n        .f2=0.0,\
                                     \n    };\
                                     \n}\
                                    \nvoid main()\
                                    \n{\
                                    \n    MyStruct const s = (MyStruct)\
                                    \n    {\
                                    \n        .f1=1,\
                                    \n        .f2=2.0,\
                                    \n    };\
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
