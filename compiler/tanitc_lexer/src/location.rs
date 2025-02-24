#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Location {
    pub row: usize,
    pub col: usize,
}

impl Location {
    pub fn new() -> Self {
        Self { row: 1, col: 1 }
    }

    pub fn new_line(&mut self) {
        self.row += 1;
        self.col = 1;
    }

    pub fn shift(&mut self) {
        self.col += 1;
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.row, self.col)
    }
}

impl Default for Location {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn location_test() {
    let mut location = Location::new();
    assert_eq!(location.row, 1);
    assert_eq!(location.col, 1);

    location.shift();
    assert_eq!(location.row, 1);
    assert_eq!(location.col, 2);

    location.new_line();
    assert_eq!(location.row, 2);
    assert_eq!(location.col, 1);

    location.new_line();
    location.new_line();
    location.shift();
    location.shift();
    assert_eq!(location.row, 4);
    assert_eq!(location.col, 3);
}
