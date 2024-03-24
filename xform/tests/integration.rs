//
// Copyright (c) 2024 Jeff Garzik
//
// This file is part of the posixutils-rs project covered under
// the MIT License.  For the full license text, please see the LICENSE
// file in the root directory of this project.
// SPDX-License-Identifier: MIT
//

use plib::{run_test, TestPlan};

fn cksum_test(test_data: &str, expected_output: &str) {
    run_test(TestPlan {
        cmd: String::from("cksum"),
        args: Vec::new(),
        stdin_data: String::from(test_data),
        expected_out: String::from(expected_output),
    });
}

#[test]
fn test_cksum() {
    cksum_test("foo\n", "3915528286 4\n");
}
