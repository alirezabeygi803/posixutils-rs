use super::{edit_script_range_data::EditScriptHunkKind, patch_format::PatchFormat};

type RangeResult = Result<Range, RangeError>;

#[derive(Clone, Copy, Debug)]
pub struct Range {
    start: usize,
    end: usize,
    kind: PatchFormat,
}

impl Range {
    pub fn new(start: usize, end: usize, kind: PatchFormat) -> Self {
        if !matches!(kind, PatchFormat::Unified) {
            assert!(
                start <= end,
                "Range creation failed! start should be less than or equal to end."
            );
        }

        Self { start, end, kind }
    }

    pub fn try_from_unified(unified_range: &str) -> RangeResult {
        let range_error = Err(RangeError::InvalidRange);
        let comma_separated_numbers = unified_range
            .chars()
            .filter(|ch| *ch != '+' && *ch != '-')
            .collect::<String>();
        let str_numbers = comma_separated_numbers.split(',');
        let mut numbers = Vec::<usize>::new();
        let radix = 10;

        for str_number in str_numbers {
            if let Ok(line_number) = usize::from_str_radix(str_number, radix) {
                numbers.push(line_number)
            } else {
                return range_error;
            }
        }

        match numbers.len() {
            1 => Ok(Range::new(numbers[0], 0, PatchFormat::Unified)),
            2 => Ok(Range::new(numbers[0], numbers[1], PatchFormat::Unified)),
            _ => range_error,
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    /// panics when self.kind is [PatchFormat::None]
    pub fn end(&self) -> usize {
        match self.kind {
            PatchFormat::None => panic!("Range should belong to one of the four formats!"),
            PatchFormat::Normal => self.end,
            PatchFormat::Unified => self.start + self.end,
            PatchFormat::Context => self.end,
            PatchFormat::EditScript => self.end,
        }
    }

    pub fn try_from_context(line: &str) -> RangeResult {
        let range_error = Err(RangeError::InvalidRange);
        let splitted = line.split(' ').collect::<Vec<&str>>();

        match splitted.len() {
            3 => {
                let range_numbers = splitted[1].split(',').collect::<Vec<&str>>();
                let radix = 10;
                let mut numbers = Vec::<usize>::new();

                if [1usize, 2usize].contains(&range_numbers.len()) {
                    for str_number in range_numbers {
                        if let Ok(number) = usize::from_str_radix(str_number, radix) {
                            numbers.push(number)
                        } else {
                            return range_error;
                        }
                    }

                    match numbers.len() {
                        1 => Ok(Range::new(numbers[0], numbers[0], PatchFormat::Context)),
                        2 => Ok(Range::new(numbers[0], numbers[1], PatchFormat::Context)),
                        _ => range_error,
                    }
                } else {
                    range_error
                }
            }
            _ => range_error,
        }
    }

    pub(crate) fn edit_script_range_kind(line: &str) -> EditScriptHunkKind {
        let last_char = line.trim().chars().last();

        if let Some(last_char) = last_char {
            return match last_char {
                'a' => EditScriptHunkKind::Insert,
                'c' => EditScriptHunkKind::Change,
                'd' => EditScriptHunkKind::Delete,
                _ => panic!("Invalid ed hunk range!"),
            };
        }

        panic!("Invalid ed hunk range!");
    }

    pub(crate) fn try_from_edit_script(line: &str) -> RangeResult {
        let comma_separated_numbers = &line.trim()[0..line.len().wrapping_sub(1)];
        let numeric_strings = comma_separated_numbers.split(',').collect::<Vec<&str>>();

        match numeric_strings.len() {
            1 => {
                let number = numeric_strings[0].parse::<usize>();

                if let Ok(number) = number {
                    return Ok(Range::new(number, number, PatchFormat::EditScript));
                }

                let error = number.unwrap_err();
                Err(RangeError::InvalidRangeWithError(error.to_string()))
            }
            2 => {
                let number1 = numeric_strings[0].parse::<usize>();
                let number2 = numeric_strings[1].parse::<usize>();

                if number1.is_ok() && number2.is_ok() {
                    let (number1, number2) = (number1.unwrap(), number2.unwrap());
                    return Ok(Range::new(number1, number2, PatchFormat::EditScript));
                }

                let mut error_data = String::new();

                if let Err(error) = number1 {
                    error_data.push_str(error.to_string().as_str());
                    error_data.push('\n');
                }

                if let Err(error) = number2 {
                    error_data.push_str(error.to_string().as_str());
                }

                Err(RangeError::InvalidRangeWithError(error_data))
            }
            _ => Err(RangeError::InvalidRange),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum RangeError {
    InvalidRange,
    InvalidRangeWithError(String),
}
