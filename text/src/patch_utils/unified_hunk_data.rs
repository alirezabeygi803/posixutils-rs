use crate::patch_utils::patch_format::PatchFormat;

use super::{patch_line::PatchLine, range::Range};

#[derive(Debug)]
pub struct UnifiedHunkData<'a> {
    f1_range: Range,
    f2_range: Range,
    lines: Vec<PatchLine<'a>>,
}

impl<'a> UnifiedHunkData<'a> {
    pub fn new(f1_range: Range, f2_range: Range, lines: Vec<PatchLine<'a>>) -> Self {
        Self {
            f1_range,
            f2_range,
            lines,
        }
    }

    pub fn lines(&self) -> &Vec<PatchLine<'a>> {
        &self.lines
    }

    pub fn lines_mut(&mut self) -> &mut Vec<PatchLine<'a>> {
        &mut self.lines
    }

    pub fn add_patch_line(&mut self, patch_line: PatchLine<'a>) {
        let patch_line_kind = patch_line.kind();
        assert!(
            matches!(patch_line_kind, PatchFormat::Unified),
            "Only Unified patch lines are allowed in UnifiedHunkData!"
        );

        self.lines.push(patch_line);
    }

    pub fn f1_range(&self) -> &Range {
        &self.f1_range
    }

    pub fn f2_range(&self) -> &Range {
        &self.f2_range
    }
}
