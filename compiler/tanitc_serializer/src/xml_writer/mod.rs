use tanitc_lexer::location::Location;
use tanitc_messages::Message;

pub mod ast;

#[derive(Clone)]
struct XmlWriterState {
    name: String,
    has_internal_tags: bool,
}

pub struct XmlWriter<'a> {
    stream: &'a mut dyn std::io::Write,
    intent: usize,
    current_state: XmlWriterState,
    cached_states: Vec<XmlWriterState>,
}

impl<'a> XmlWriter<'a> {
    pub fn new(stream: &'a mut dyn std::io::Write) -> Result<Self, &'static str> {
        Ok(Self {
            stream,
            intent: 0,
            current_state: XmlWriterState {
                name: "".to_string(),
                has_internal_tags: false,
            },
            cached_states: Vec::new(),
        })
    }

    pub fn serialize_err(e: std::io::Error) -> Message {
        Message {
            location: Location::default(),
            text: format!("XML serialization error: {e}"),
        }
    }
}

// public methods
impl XmlWriter<'_> {
    pub fn begin_tag(&mut self, name: &str) -> Result<(), Message> {
        if let Err(e) = self.begin_tag_internal(name) {
            Err(Self::serialize_err(e))
        } else {
            Ok(())
        }
    }

    pub fn end_tag(&mut self) -> Result<(), Message> {
        if let Err(e) = self.end_tag_internal() {
            Err(Self::serialize_err(e))
        } else {
            Ok(())
        }
    }

    pub fn put_param<V: std::fmt::Display>(&mut self, key: &str, value: V) -> Result<(), Message> {
        if let Err(e) = self.put_param_internal(key, value) {
            Err(Self::serialize_err(e))
        } else {
            Ok(())
        }
    }

    pub fn close(&mut self) {
        let _ = writeln!(self.stream);
    }
}

// Private methods
impl XmlWriter<'_> {
    fn begin_tag_internal(&mut self, name: &str) -> std::io::Result<()> {
        if !self.cached_states.is_empty() && !self.current_state.has_internal_tags {
            /* close previous tag */
            write!(self.stream, ">")?;
        }
        writeln!(self.stream)?;

        /* Save previous state */
        self.current_state.has_internal_tags = true;
        self.cached_states.push(self.current_state.clone());

        /* Set new state */
        self.current_state.has_internal_tags = false;
        self.current_state.name = name.to_string();

        self.put_intent()?;
        write!(self.stream, "<{name}")?;

        self.increase_intent();

        Ok(())
    }

    fn end_tag_internal(&mut self) -> std::io::Result<()> {
        self.decrease_intent();
        if self.current_state.has_internal_tags {
            writeln!(self.stream)?;
            self.put_intent()?;
            write!(self.stream, "</{}>", self.current_state.name)?;
        } else {
            write!(self.stream, "/>")?;
        }

        self.current_state = self.cached_states.pop().unwrap();

        Ok(())
    }

    fn put_param_internal<V: std::fmt::Display>(
        &mut self,
        key: &str,
        value: V,
    ) -> std::io::Result<()> {
        if self.current_state.has_internal_tags {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Parameters aren\'t allowed after internal tags",
            ));
        }
        write!(self.stream, " {key}=\"{value}\"")?;

        Ok(())
    }

    fn put_intent(&mut self) -> std::io::Result<()> {
        for _ in 0..self.intent {
            write!(self.stream, "    ")?;
        }

        Ok(())
    }

    fn increase_intent(&mut self) {
        self.intent += 1;
    }

    fn decrease_intent(&mut self) {
        self.intent -= 1;
    }
}
