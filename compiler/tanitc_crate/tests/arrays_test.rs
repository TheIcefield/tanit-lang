use tanitc_analyzer::Analyzer;
use tanitc_codegen::c_generator::CodeGenStream;
use tanitc_parser::Parser;

use pretty_assertions::assert_str_eq;

#[test]
fn array_work_test() {
    const SRC_TEXT: &str = "\nfunc main() {\
                            \n    var mut arr_1: [f32: 6]\
                            \n    var arr_2: [i32: 3] = [4, 5, 6]\
                            \n    var arr_3 = [1.0, 2.0, 3.0]\
                            \n    arr_1[1 + 1] = 7.0\
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
        const HEADER_EXPECTED: &str = "void main();\n";

        const SOURCE_EXPECTED: &str = "void main()\
                                     \n{\
                                     \n    float arr_1[6];\
                                     \n    signed int const arr_2[3] = { 4, 5, 6 };\
                                     \n    float const arr_3[3] = { 1, 2, 3 };\
                                     \n    arr_1[1 + 1] = 7;\
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
fn immutable_array_bad_test() {
    const SRC_TEXT: &str = "\nfunc main() {\
                            \n    var arr = [1.0, 2.0, 3.0] # immutable\
                            \n    arr[0] = 7.0\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Failed to create parser");

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }
    }

    {
        const EXPECTED_ERR: &str = "Semantic error: Cannot mutate immutable object of type \"f32\" is immutable in current scope";

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 1);
        assert_str_eq!(errors[0].text, EXPECTED_ERR);
    }
}

#[test]
fn strange_index_array_bad_test() {
    const SRC_TEXT: &str = "\nfunc main() {\
                            \n    var mut arr = [1.0, 2.0, 3.0]\
                            \n    arr[3.14] = 7.0\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Failed to create parser");

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }
    }

    {
        const EXPECTED_ERR: &str = "Semantic error: Invalid index type: f32";

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 1);
        assert_str_eq!(errors[0].text, EXPECTED_ERR);
    }
}
