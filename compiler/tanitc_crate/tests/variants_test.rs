use tanitc_analyzer::Analyzer;
use tanitc_codegen::c_generator::CodeGenStream;
use tanitc_lexer::Lexer;
use tanitc_options::CompileOptions;
use tanitc_parser::Parser;
use tanitc_serializer::xml_writer::XmlWriter;

use pretty_assertions::assert_str_eq;

#[test]
fn variant_work_test() {
    const SRC_TEXT: &str = "\npub variant MyVariant\
                            \n{\
                            \n    f1\
                            \n    f2(i32, i32)\
                            \n    f3 {\
                            \n        x: i32\
                            \n        y: f32\
                            \n    }\
                            \n}\
                            \nfunc main() {\
                            \n    var v1 = MyVariant::f1\
                            \n    var v3 = MyVariant::f3 {\
                            \n                 x: 4,\
                            \n                 y: 7.5\
                            \n             }\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Failed to create lexer"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }
    }

    {
        let compile_options = CompileOptions {
            allow_variants: true,
            ..Default::default()
        };

        let mut analyzer = Analyzer::with_options(compile_options);
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:?}", analyzer.get_errors());
        }
    }

    {
        const EXPECTED: &str = "\n<variant-definition name=\"MyVariant\">\
                                \n    <attributes publicity=\"Public\"/>\
                                \n    <field name=\"f1\"/>\
                                \n    <field name=\"f2\">\
                                \n        <type style=\"primitive\" name=\"i32\"/>\
                                \n        <type style=\"primitive\" name=\"i32\"/>\
                                \n    </field>\
                                \n    <field name=\"f3\">\
                                \n        <field name=\"x\">\
                                \n            <type style=\"primitive\" name=\"i32\"/>\
                                \n        </field>\
                                \n        <field name=\"y\">\
                                \n            <type style=\"primitive\" name=\"f32\"/>\
                                \n        </field>\
                                \n    </field>\
                                \n</variant-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"v1\" is-global=\"false\" mutability=\"Immutable\">\
                                \n            <type style=\"named\" name=\"MyVariant\"/>\
                                \n        </variable-definition>\
                                \n        <operation>\
                                \n            <struct-initialization name=\"MyVariant\">\
                                \n                <field name=\"__kind__\">\
                                \n                    <identifier name=\"__MyVariant__kind__f1__\"/>\
                                \n                </field>\
                                \n                <field name=\"__data__\">\
                                \n                    <struct-initialization name=\"__MyVariant__data__\">\
                                \n                        <field name=\"f1\">\
                                \n                            <struct-initialization name=\"__MyVariant__data__f1__\"/>\
                                \n                        </field>\
                                \n                    </struct-initialization>\
                                \n                </field>\
                                \n            </struct-initialization>\
                                \n        </operation>\
                                \n    </operation>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"v3\" is-global=\"false\" mutability=\"Immutable\">\
                                \n            <type style=\"named\" name=\"MyVariant\"/>\
                                \n        </variable-definition>\
                                \n        <operation>\
                                \n            <struct-initialization name=\"MyVariant\">\
                                \n                <field name=\"__kind__\">\
                                \n                    <identifier name=\"__MyVariant__kind__f3__\"/>\
                                \n                </field>\
                                \n                <field name=\"__data__\">\
                                \n                    <struct-initialization name=\"__MyVariant__data__\">\
                                \n                        <field name=\"f3\">\
                                \n                            <struct-initialization name=\"__MyVariant__data__f3__\">\
                                \n                                <field name=\"x\">\
                                \n                                    <literal style=\"integer-number\" value=\"4\"/>\
                                \n                                </field>\
                                \n                                <field name=\"y\">\
                                \n                                    <literal style=\"decimal-number\" value=\"7.5\"/>\
                                \n                                </field>\
                                \n                            </struct-initialization>\
                                \n                        </field>\
                                \n                    </struct-initialization>\
                                \n                </field>\
                                \n            </struct-initialization>\
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
                                     \n    __MyVariant__kind__f1__,\
                                     \n    __MyVariant__kind__f2__,\
                                     \n    __MyVariant__kind__f3__,\
                                     \n} __MyVariant__kind__;\
                                     \n\
                                     \ntypedef struct { } __MyVariant__data__f1__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n    signed int _0;\
                                     \n    signed int _1;\
                                     \n} __MyVariant__data__f2__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n    signed int x;\
                                     \n    float y;\
                                     \n} __MyVariant__data__f3__;\
                                     \n\
                                     \ntypedef union __MyVariant__data__ {\
                                     \n    __MyVariant__data__f1__ f1;\
                                     \n    __MyVariant__data__f2__ f2;\
                                     \n    __MyVariant__data__f3__ f3;\
                                     \n} __MyVariant__data__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n    __MyVariant__kind__ __kind__;\
                                     \n    __MyVariant__data__ __data__;\
                                     \n} MyVariant;\
                                     \n\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main(){\
                                        \nMyVariant const v1 = (MyVariant){\
                                            \n.__kind__=__MyVariant__kind__f1__,\
                                            \n.__data__=(__MyVariant__data__){\
                                               \n.f1=(__MyVariant__data__f1__){\
                                                   \n},\
                                               \n},\
                                            \n};\
                                            \nMyVariant const v3 = (MyVariant){\
                                                \n.__kind__=__MyVariant__kind__f3__,\
                                                \n.__data__=(__MyVariant__data__){\
                                                    \n.f3=(__MyVariant__data__f3__){\
                                                        \n.x=4,\
                                                        \n.y=7.5,\
                                                    \n},\
                                                \n},\
                                            \n};\
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
fn variant_in_module_work_test() {
    const SRC_TEXT: &str = "\nmodule math {\
                            \n    variant MyVariant\
                            \n    {\
                            \n        f1\
                            \n        f2(i32, i32)\
                            \n        f3 {\
                            \n            x: i32\
                            \n            y: f32\
                            \n        }\
                            \n    }\
                            \n}\
                            \nfunc main() {\
                            \n    var v1 = math::MyVariant::f1\
                            \n    var v3 = math::MyVariant::f3 {\
                            \n                 x: 4,\
                            \n                 y: 7.5\
                            \n             }\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Failed to create lexer"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }
    }

    {
        let compile_options = CompileOptions {
            allow_variants: true,
            ..Default::default()
        };

        let mut analyzer = Analyzer::with_options(compile_options);
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:?}", analyzer.get_errors());
        }
    }

    {
        const EXPECTED: &str = "\n<module-definition name=\"math\">\
                                \n    <variant-definition name=\"MyVariant\">\
                                \n        <field name=\"f1\"/>\
                                \n        <field name=\"f2\">\
                                \n            <type style=\"primitive\" name=\"i32\"/>\
                                \n            <type style=\"primitive\" name=\"i32\"/>\
                                \n        </field>\
                                \n        <field name=\"f3\">\
                                \n            <field name=\"x\">\
                                \n                <type style=\"primitive\" name=\"i32\"/>\
                                \n            </field>\
                                \n            <field name=\"y\">\
                                \n                <type style=\"primitive\" name=\"f32\"/>\
                                \n            </field>\
                                \n        </field>\
                                \n    </variant-definition>\
                                \n</module-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"v1\" is-global=\"false\" mutability=\"Immutable\">\
                                \n            <type style=\"named\" name=\"MyVariant\"/>\
                                \n        </variable-definition>\
                                \n        <operation>\
                                \n            <struct-initialization name=\"MyVariant\">\
                                \n                <field name=\"__kind__\">\
                                \n                    <identifier name=\"__MyVariant__kind__f1__\"/>\
                                \n                </field>\
                                \n                <field name=\"__data__\">\
                                \n                    <struct-initialization name=\"__MyVariant__data__\">\
                                \n                        <field name=\"f1\">\
                                \n                            <struct-initialization name=\"__MyVariant__data__f1__\"/>\
                                \n                        </field>\
                                \n                    </struct-initialization>\
                                \n                </field>\
                                \n            </struct-initialization>\
                                \n        </operation>\
                                \n    </operation>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"v3\" is-global=\"false\" mutability=\"Immutable\">\
                                \n            <type style=\"named\" name=\"MyVariant\"/>\
                                \n        </variable-definition>\
                                \n        <operation>\
                                \n            <struct-initialization name=\"MyVariant\">\
                                \n                <field name=\"__kind__\">\
                                \n                    <identifier name=\"__MyVariant__kind__f3__\"/>\
                                \n                </field>\
                                \n                <field name=\"__data__\">\
                                \n                    <struct-initialization name=\"__MyVariant__data__\">\
                                \n                        <field name=\"f3\">\
                                \n                            <struct-initialization name=\"__MyVariant__data__f3__\">\
                                \n                                <field name=\"x\">\
                                \n                                    <literal style=\"integer-number\" value=\"4\"/>\
                                \n                                </field>\
                                \n                                <field name=\"y\">\
                                \n                                    <literal style=\"decimal-number\" value=\"7.5\"/>\
                                \n                                </field>\
                                \n                            </struct-initialization>\
                                \n                        </field>\
                                \n                    </struct-initialization>\
                                \n                </field>\
                                \n            </struct-initialization>\
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
                                     \n    __MyVariant__kind__f1__,\
                                     \n    __MyVariant__kind__f2__,\
                                     \n    __MyVariant__kind__f3__,\
                                     \n} __MyVariant__kind__;\
                                     \n\
                                     \ntypedef struct { } __MyVariant__data__f1__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n    signed int _0;\
                                     \n    signed int _1;\
                                     \n} __MyVariant__data__f2__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n    signed int x;\
                                     \n    float y;\
                                     \n} __MyVariant__data__f3__;\
                                     \n\
                                     \ntypedef union __MyVariant__data__ {\
                                     \n    __MyVariant__data__f1__ f1;\
                                     \n    __MyVariant__data__f2__ f2;\
                                     \n    __MyVariant__data__f3__ f3;\
                                     \n} __MyVariant__data__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n    __MyVariant__kind__ __kind__;\
                                     \n    __MyVariant__data__ __data__;\
                                     \n} MyVariant;\
                                     \n\
                                     \nvoid main();\n";

        // TODO: correct data field initialization
        const SOURCE_EXPECTED: &str = "void main(){\
                                        \nMyVariant const v1 = (MyVariant){\
                                            \n.__kind__=__MyVariant__kind__f1__,\
                                            \n.__data__=(__MyVariant__data__){\
                                                \n.f1=(__MyVariant__data__f1__){\
                                                \n},\
                                            \n},\
                                        \n};\
                                        \nMyVariant const v3 = (MyVariant){\
                                            \n.__kind__=__MyVariant__kind__f3__,\
                                            \n.__data__=(__MyVariant__data__){\
                                                \n.f3=(__MyVariant__data__f3__){\
                                                    \n.x=4,\
                                                    \n.y=7.5,\
                                                \n},\
                                            \n},\
                                        \n};\
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
fn denied_variants_test() {
    const SRC_TEXT: &str = "\nvariant MyVariant\
                            \n{\
                            \n    f1\
                            \n    f2(i32, i32)\
                            \n    f3 {\
                            \n        x: i32\
                            \n        y: f32\
                            \n    }\
                            \n}\
                            \nfunc main() {\
                            \n    var v1 = MyVariant::f1\
                            \n    var v3 = MyVariant::f3 {\
                            \n                 x: 4,\
                            \n                 y: 7.5\
                            \n             }\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Failed to create lexer"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }
    }

    {
        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        assert!(analyzer.has_errors());
    }
}
