// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar_spec_model::vocabulary::speller::Speller;
use beggar_spec_model::vocabulary::spelling::Spelling;

const SPELLING: Spelling<Colour> = Spelling {
    kind: "a colour",
    words: &[(Colour::Red, "Red"), (Colour::Green, "Green")],
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Colour {
    Red,
    Green,
    Blue,
}

#[test]
fn name_of_a_word_is_its_spelling() {
    // Arrange & Act
    let name = Speller::name(&SPELLING, Colour::Green);

    // Assert
    assert_eq!(name, "Green");
}

#[test]
#[should_panic(expected = "a colour")]
fn name_of_a_word_the_spelling_leaves_out_panics_naming_its_kind() {
    // Arrange & Act & Assert
    let _ = Speller::name(&SPELLING, Colour::Blue);
}

#[test]
fn of_a_spelling_is_its_word() {
    // Arrange & Act
    let word = Speller::of(&SPELLING, "Red");

    // Assert
    assert_eq!(word, Colour::Red);
}

#[test]
#[should_panic(expected = "a colour")]
fn of_a_spelling_no_word_has_panics_naming_its_kind() {
    // Arrange & Act & Assert
    let _ = Speller::of(&SPELLING, "Blue");
}
