use tanitc_ident::Ident;

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name {
    pub id: Ident,
    pub prefix: Vec<Ident>,
}

impl Name {
    pub fn short_name(&self) -> String {
        self.id.to_string()
    }

    pub fn full_name(&self) -> String {
        let mut s = String::new();

        for id in self.prefix.iter() {
            s.push_str(&id.to_string());
            s.push_str("__");
        }

        s.push_str(&self.short_name());

        s
    }
}
