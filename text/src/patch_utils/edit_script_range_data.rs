use super::range::Range;

#[derive(Debug, Clone, Copy)]
pub enum EditScriptHunkKind {
    Insert,
    Delete,
    Change,
}

#[derive(Debug, Clone)]
pub struct EditScriptRangeData<'a> {
    line: &'a str,
    #[allow(dead_code)]
    range: Range,
    kind: EditScriptHunkKind,
}

impl<'a> EditScriptRangeData<'a> {
    pub fn new(line: &'a str, range: Range, kind: EditScriptHunkKind) -> Self {
        Self { line, range, kind }
    }

    pub fn line(&self) -> &str {
        self.line
    }

    pub fn kind(&self) -> EditScriptHunkKind {
        self.kind
    }

    pub fn range(&self) -> Range {
        self.range
    }
}