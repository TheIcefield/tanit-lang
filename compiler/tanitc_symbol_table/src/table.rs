use std::collections::{BTreeMap, LinkedList};

use tanitc_ast::attributes::Safety;
use tanitc_ident::Ident;

use crate::entry::SymbolKind;

use super::entry::Entry;

#[derive(Debug, Clone)]
pub struct Table {
    entries: BTreeMap<Ident, Entry>,
    stack: LinkedList<Table>,
    safety: Safety,
}

impl Table {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            stack: LinkedList::new(),
            safety: Safety::Safe,
        }
    }
}

impl Table {
    pub fn enter_scope(&mut self, safety: Safety) {
        let mut table = Table::new();
        table.safety = safety;

        self.stack.push_back(table);
    }

    pub fn exit_scope(&mut self) {
        self.stack.pop_back();
    }

    pub fn insert(&mut self, entry: Entry) {
        if let Some(back) = self.stack.back_mut() {
            back.entries.insert(entry.name, entry);
        } else {
            self.entries.insert(entry.name, entry);
        }
    }

    pub fn lookup(&self, name: Ident) -> Option<&Entry> {
        let mut res: Option<&Entry> = self.entries.get(&name);

        for scope in self.stack.iter().rev() {
            let entry = scope.entries.get(&name);
            if entry.is_some() {
                res = entry;
            }
        }

        res
    }

    pub fn lookup_mut(&mut self, name: Ident) -> Option<&mut Entry> {
        let mut res: Option<&mut Entry> = self.entries.get_mut(&name);

        for scope in self.stack.iter_mut().rev() {
            let entry = scope.entries.get_mut(&name);
            if entry.is_some() {
                res = entry;
            }
        }

        res
    }

    pub fn lookup_qualified(
        &self,
        mut names: std::iter::Peekable<std::slice::Iter<Ident>>,
    ) -> Option<&Entry> {
        let next = names.next();
        if next.is_none() {
            return None;
        }

        let entry = self.lookup(*next.unwrap());
        if entry.is_none() {
            return None;
        }

        // If it was not the last
        if names.peek().is_some() {
            let entry = entry.unwrap();
            if let SymbolKind::Module { ref table } = &entry.kind {
                // lookup qualified in module
                table.lookup_qualified(names)
            } else {
                // lookup in self
                self.lookup(*names.next().unwrap())
            }
        } else {
            entry
        }
    }
}

#[test]
fn table_test() {
    /* example:
     * Module Main {       # Main: @s
     *     func bar() { }  # bar:  @s/Main
     *     func main() {   # main: @s/Main
     *         let var = 5 # var:  @s/Main/main
     *     }
     * }
     */
    use tanitc_ty::Type;

    use crate::entry::{SymbolKind, VarStorageType};

    let main_mod_id = Ident::from("Main".to_string());
    let main_fn_id = Ident::from("main".to_string());
    let bar_id = Ident::from("bar".to_string());
    let var_id = Ident::from("var".to_string());
    let baz_id = Ident::from("baz".to_string());

    let mut table = Table::new();

    table.insert(Entry {
        name: main_mod_id,
        is_static: true,
        kind: SymbolKind::Module {
            table: Box::new(Table::new()),
        },
    });

    let main_mod = table.lookup_mut(main_mod_id).unwrap();
    let SymbolKind::Module { ref mut table } = &mut main_mod.kind else {
        unreachable!()
    };

    {
        table.enter_scope(Safety::Safe); // enter Main

        table.insert(Entry {
            name: bar_id,
            is_static: false,
            kind: SymbolKind::Func {
                parameters: vec![],
                return_type: Type::unit(),
                is_virtual: false,
                is_inline: false,
                no_return: true,
            },
        });

        {
            table.enter_scope(Safety::Safe); // enter bar

            // check if var not visible in bar
            assert!(table.lookup(var_id).is_none());

            // check if baz not defined in bar
            assert!(table.lookup(baz_id).is_none());

            table.exit_scope(); // exit bar
        }

        table.insert(Entry {
            name: main_fn_id,
            is_static: false,
            kind: SymbolKind::Func {
                parameters: vec![],
                return_type: Type::unit(),
                is_virtual: false,
                is_inline: false,
                no_return: true,
            },
        });

        {
            table.enter_scope(Safety::Safe); // enter main

            table.insert(Entry {
                name: var_id,
                is_static: false,
                kind: SymbolKind::Var {
                    storage: VarStorageType::Auto,
                    offset: 0,
                    size: 0,
                },
            });

            // check if var visible in main
            assert!(table.lookup(var_id).is_some());

            table.exit_scope(); // exit main
        }

        // check if main visible in Main
        assert!(table.lookup(main_fn_id).is_some());

        // check if var not visible in Main
        assert!(table.lookup(var_id).is_none());

        // check if baz not visible in Main
        assert!(table.lookup(baz_id).is_none());
    }
}

#[test]
fn qualified_symbol_test() {
    /* example:
     * Module M1 {       # M1
     *     func f1() { } # M1/f1
     *     func f2() { } # M1/f2
     * }
     * Module M2 {       # M2
     *     func f2() { } # M2/f2
     * }
     */

    use crate::entry::SymbolKind;
    use tanitc_ty::Type;

    let m1_id = Ident::from("M1".to_string());
    let m2_id = Ident::from("M2".to_string());
    let f1_id = Ident::from("f1".to_string());
    let f2_id = Ident::from("f2".to_string());

    let mut table = Table::new();

    table.insert(Entry {
        name: m1_id,
        is_static: true,
        kind: SymbolKind::Module {
            table: Box::new(Table::new()),
        },
    });

    {
        let m1 = table.lookup_mut(m1_id).unwrap();
        let SymbolKind::Module { ref mut table } = &mut m1.kind else {
            unreachable!()
        };

        table.insert(Entry {
            name: f1_id,
            is_static: false,
            kind: SymbolKind::Func {
                parameters: vec![],
                return_type: Type::unit(),
                is_virtual: false,
                is_inline: false,
                no_return: true,
            },
        });

        table.insert(Entry {
            name: f2_id,
            is_static: false,
            kind: SymbolKind::Func {
                parameters: vec![],
                return_type: Type::unit(),
                is_virtual: false,
                is_inline: false,
                no_return: true,
            },
        });
    }

    table.insert(Entry {
        name: m2_id,
        is_static: true,
        kind: SymbolKind::Module {
            table: Box::new(Table::new()),
        },
    });

    {
        let m2 = table.lookup_mut(m2_id).unwrap();
        let SymbolKind::Module { ref mut table } = &mut m2.kind else {
            unreachable!()
        };

        table.insert(Entry {
            name: f2_id,
            is_static: false,
            kind: SymbolKind::Func {
                parameters: vec![],
                return_type: Type::unit(),
                is_virtual: false,
                is_inline: false,
                no_return: true,
            },
        });
    }

    assert!(table
        .lookup_qualified([m1_id, f2_id].iter().peekable())
        .is_some());
    assert!(table
        .lookup_qualified([m2_id, f1_id].iter().peekable())
        .is_none());
}
