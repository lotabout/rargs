extern crate assert_cli;

static RARGS: &'static str = "./target/release/rargs";

#[test]
fn regex_should_match() {
    assert_cli::Assert::command(&[
        RARGS,
        "-p",
        r"^(?P<year>\d{4})-(\d{2})-(\d{2})$",
        "echo",
        "{1} {2} {3}",
    ])
    .stdin("2018-01-20")
    .stdout()
    .is("2018 01 20")
    .unwrap();
}

#[test]
fn test_regex_group_name_should_match() {
    assert_cli::Assert::command(&[
        RARGS,
        "-p",
        "^(?P<year>\\d{4})-(\\d{2})-(\\d{2})$",
        "echo",
        "{year} {2} {3}",
    ])
    .stdin("2018-01-20")
    .stdout()
    .is("2018 01 20")
    .unwrap();
}

#[test]
fn test_negtive_regex_group_should_work() {
    assert_cli::Assert::command(&[
        RARGS,
        "-p",
        "^(?P<year>\\d{4})-(\\d{2})-(\\d{2})$",
        "echo",
        "{-3} {-2} {-1}",
    ])
    .stdin("2018-01-20")
    .stdout()
    .is("2018 01 20")
    .unwrap();
}

#[test]
fn test_read0_short() {
    assert_cli::Assert::command(&[RARGS, "-0", "echo", "{}"])
        .stdin("a\0b")
        .stdout()
        .is("a\nb")
        .unwrap();
}

#[test]
fn test_read0_long() {
    assert_cli::Assert::command(&[RARGS, "--read0", "echo", "{}"])
        .stdin("a\0b")
        .stdout()
        .is("a\nb")
        .unwrap();
}

#[test]
fn test_no_read0() {
    assert!(assert_cli::Assert::command(&[RARGS, "echo", "{}"])
        .stdin("a\0b")
        .stdout()
        .is("a\0b")
        .execute()
        .is_err());
}

#[test]
fn test_default_delimiter() {
    assert_cli::Assert::command(&[RARGS, "echo", "X{1},{2},{3}X"])
        .stdin("a b  c")
        .stdout()
        .is("Xa,b,cX")
        .unwrap();
}

#[test]
fn test_delimiter() {
    assert_cli::Assert::command(&[RARGS, "-d", ",", "echo", "X{1},{2},{3},{4}X"])
        .stdin("a,b,,c")
        .stdout()
        .is("Xa,b,,cX")
        .unwrap();
}

#[test]
fn test_range_left_inf() {
    assert_cli::Assert::command(&[RARGS, "-d", ",", "echo", "X{..3}X"])
        .stdin("1,2,3,4,5,6")
        .stdout()
        .is("X1 2 3X")
        .unwrap();

    assert_cli::Assert::command(&[RARGS, "-d", ",", "echo", "X{..-2}X"])
        .stdin("1,2,3,4,5,6")
        .stdout()
        .is("X1 2 3 4 5X")
        .unwrap();

    assert_cli::Assert::command(&[RARGS, "-d", ",", "echo", "X{..0}X"])
        .stdin("1,2,3,4,5,6")
        .stdout()
        .is("XX")
        .unwrap();
}

#[test]
fn test_range_right_inf() {
    assert_cli::Assert::command(&[RARGS, "-d", ",", "echo", "X{3..}X"])
        .stdin("1,2,3,4,5,6")
        .stdout()
        .is("X3 4 5 6X")
        .unwrap();

    assert_cli::Assert::command(&[RARGS, "-d", ",", "echo", "X{-2..}X"])
        .stdin("1,2,3,4,5,6")
        .stdout()
        .is("X5 6X")
        .unwrap();

    assert_cli::Assert::command(&[RARGS, "-d", ",", "echo", "X{7..}X"])
        .stdin("1,2,3,4,5,6")
        .stdout()
        .is("XX")
        .unwrap();
}

#[test]
fn test_range_both() {
    assert_cli::Assert::command(&[RARGS, "-d", ",", "echo", "X{3..3}X"])
        .stdin("1,2,3,4,5,6")
        .stdout()
        .is("X3X")
        .unwrap();

    assert_cli::Assert::command(&[RARGS, "-d", ",", "echo", "X{3..4}X"])
        .stdin("1,2,3,4,5,6")
        .stdout()
        .is("X3 4X")
        .unwrap();

    assert_cli::Assert::command(&[RARGS, "-d", ",", "echo", "X{3..7}X"])
        .stdin("1,2,3,4,5,6")
        .stdout()
        .is("X3 4 5 6X")
        .unwrap();

    assert_cli::Assert::command(&[RARGS, "-d", ",", "echo", "X{4..3}X"])
        .stdin("1,2,3,4,5,6")
        .stdout()
        .is("XX")
        .unwrap();
}

#[test]
fn test_field_separator() {
    assert_cli::Assert::command(&[RARGS, "-d", ",", "echo", "X{3..4:_}X"])
        .stdin("1,2,3,4,5,6")
        .stdout()
        .is("X3_4X")
        .unwrap();

    assert_cli::Assert::command(&[RARGS, "-d", ",", "echo", "X{3..4:-}X"])
        .stdin("1,2,3,4,5,6")
        .stdout()
        .is("X3-4X")
        .unwrap();
}

#[test]
fn test_global_field_separator() {
    assert_cli::Assert::command(&[RARGS, "-d", ",", "-s", "/", "echo", "X{3..4}X"])
        .stdin("1,2,3,4,5,6")
        .stdout()
        .is("X3/4X")
        .unwrap();

    assert_cli::Assert::command(&[RARGS, "-d", ",", "-s", "/", "echo", "X{3..4:,}X"])
        .stdin("1,2,3,4,5,6")
        .stdout()
        .is("X3,4X")
        .unwrap();
}

#[test]
fn test_line_num_should_work() {
    assert_cli::Assert::command(&[RARGS, "echo", "{LN} {}"])
        .stdin("line 1\nline 2")
        .stdout()
        .is("1 line 1\n2 line 2")
        .unwrap();

    assert_cli::Assert::command(&[RARGS, "echo", "{LINENUM} {}"])
        .stdin("line 1\nline 2")
        .stdout()
        .is("1 line 1\n2 line 2")
        .unwrap();
}

#[test]
fn test_start_num_should_be_working() {
    assert_cli::Assert::command(&[RARGS, "-n", "10", "echo", "{LN} {}"])
        .stdin("line 1\nline 2")
        .stdout()
        .is("10 line 1\n11 line 2")
        .unwrap();
}
