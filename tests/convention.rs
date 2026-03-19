//! Unit tests for [`Convention`] — parsing, display, and stem validation.

use convention_lint::convention::{Convention, UnknownConvention};

// ---------------------------------------------------------------------------
// FromStr
// ---------------------------------------------------------------------------

#[test]
fn parse_all_known_variants() {
    let cases: &[(&str, Convention)] = &[
        ("snake_case", Convention::SnakeCase),
        ("CamelCase", Convention::CamelCase),
        ("PascalCase", Convention::CamelCase), // accepted alias
        ("camelCase", Convention::LowerCamelCase),
        ("SCREAMING_SNAKE_CASE", Convention::ScreamingSnakeCase),
        ("kebab-case", Convention::KebabCase),
    ];
    for &(input, ref expected) in cases {
        let parsed: Convention = input
            .parse()
            .unwrap_or_else(|_| panic!("failed to parse `{input}`"));
        assert_eq!(&parsed, expected, "input: `{input}`");
    }
}

#[test]
fn parse_unknown_returns_err() {
    let bad = ["UNKNOWN", "", "snake-case", "camel_case", "Pascal_Case"];
    for input in bad {
        let result = input.parse::<Convention>();
        assert!(
            result.is_err(),
            "`{input}` should have been rejected but parsed successfully"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, UnknownConvention(_)),
            "unexpected error type for `{input}`"
        );
    }
}

// ---------------------------------------------------------------------------
// Display / as_str round-trip
// ---------------------------------------------------------------------------

#[test]
fn display_roundtrip() {
    let variants = [
        Convention::SnakeCase,
        Convention::CamelCase,
        Convention::LowerCamelCase,
        Convention::ScreamingSnakeCase,
        Convention::KebabCase,
    ];
    for variant in variants {
        let s = variant.to_string();
        let parsed: Convention = s
            .parse()
            .unwrap_or_else(|_| panic!("`{s}` could not be round-tripped"));
        assert_eq!(parsed, variant, "round-trip failed for `{s}`");
    }
}

#[test]
fn as_str_matches_display() {
    let variants = [
        Convention::SnakeCase,
        Convention::CamelCase,
        Convention::LowerCamelCase,
        Convention::ScreamingSnakeCase,
        Convention::KebabCase,
    ];
    for variant in &variants {
        assert_eq!(variant.as_str(), variant.to_string().as_str());
    }
}

// ---------------------------------------------------------------------------
// is_valid — snake_case
// ---------------------------------------------------------------------------

mod snake_case {
    use convention_lint::Convention;
    const C: Convention = Convention::SnakeCase;

    #[test]
    fn valid_stems() {
        let valid = ["hello", "hello_world", "foo123", "a", "a1_b2"];
        for stem in valid {
            assert!(C.is_valid(stem), "`{stem}` should be valid snake_case");
        }
    }

    #[test]
    fn invalid_stems() {
        let invalid = [
            ("", "empty"),
            ("Hello", "uppercase first"),
            ("helloWorld", "camelCase"),
            ("hello__world", "double underscore"),
            ("hello_", "trailing underscore"),
            ("_hello", "leading underscore"),
            ("hello world", "space"),
            ("hello-world", "hyphen"),
        ];
        for (stem, reason) in invalid {
            assert!(
                !C.is_valid(stem),
                "`{stem}` should NOT be valid snake_case ({reason})"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// is_valid — CamelCase
// ---------------------------------------------------------------------------

mod camel_case {
    use convention_lint::Convention;
    const C: Convention = Convention::CamelCase;

    #[test]
    fn valid_stems() {
        let valid = ["MyService", "Foo", "FooBar", "A", "Foo123"];
        for stem in valid {
            assert!(C.is_valid(stem), "`{stem}` should be valid CamelCase");
        }
    }

    #[test]
    fn invalid_stems() {
        let invalid = [
            ("", "empty"),
            ("my_service", "snake_case"),
            ("myService", "lowerCamelCase"),
            ("My_Service", "underscore"),
            ("My-Service", "hyphen"),
        ];
        for (stem, reason) in invalid {
            assert!(
                !C.is_valid(stem),
                "`{stem}` should NOT be valid CamelCase ({reason})"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// is_valid — lowerCamelCase
// ---------------------------------------------------------------------------

mod lower_camel_case {
    use convention_lint::Convention;
    const C: Convention = Convention::LowerCamelCase;

    #[test]
    fn valid_stems() {
        let valid = ["myService", "foo", "fooBar", "a", "foo123"];
        for stem in valid {
            assert!(C.is_valid(stem), "`{stem}` should be valid camelCase");
        }
    }

    #[test]
    fn invalid_stems() {
        let invalid = [
            ("", "empty"),
            ("MyService", "UpperCamelCase"),
            ("my_service", "snake with underscore"),
            ("my-service", "kebab"),
        ];
        for (stem, reason) in invalid {
            assert!(
                !C.is_valid(stem),
                "`{stem}` should NOT be valid camelCase ({reason})"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// is_valid — SCREAMING_SNAKE_CASE
// ---------------------------------------------------------------------------

mod screaming_snake_case {
    use convention_lint::Convention;
    const C: Convention = Convention::ScreamingSnakeCase;

    #[test]
    fn valid_stems() {
        let valid = ["MY_CONST", "FOO", "FOO_BAR", "A", "FOO_123"];
        for stem in valid {
            assert!(
                C.is_valid(stem),
                "`{stem}` should be valid SCREAMING_SNAKE_CASE"
            );
        }
    }

    #[test]
    fn invalid_stems() {
        let invalid = [
            ("", "empty"),
            ("my_const", "lowercase"),
            ("My_Const", "mixed case"),
            ("FOO__BAR", "double underscore"),
            ("FOO_", "trailing underscore"),
        ];
        for (stem, reason) in invalid {
            assert!(
                !C.is_valid(stem),
                "`{stem}` should NOT be valid SCREAMING_SNAKE_CASE ({reason})"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// is_valid — kebab-case
// ---------------------------------------------------------------------------

mod kebab_case {
    use convention_lint::Convention;
    const C: Convention = Convention::KebabCase;

    #[test]
    fn valid_stems() {
        let valid = ["my-service", "foo", "foo-bar", "a", "foo-123"];
        for stem in valid {
            assert!(C.is_valid(stem), "`{stem}` should be valid kebab-case");
        }
    }

    #[test]
    fn invalid_stems() {
        let invalid = [
            ("", "empty"),
            ("My-Service", "uppercase"),
            ("my_service", "underscore"),
            ("my--service", "double hyphen"),
            ("my-service-", "trailing hyphen"),
        ];
        for (stem, reason) in invalid {
            assert!(
                !C.is_valid(stem),
                "`{stem}` should NOT be valid kebab-case ({reason})"
            );
        }
    }
}
