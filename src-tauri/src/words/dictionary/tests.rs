use super::*;

const REPLY: &str = r#"[
  {
    "word": "ephemeral",
    "phonetic": "",
    "phonetics": [
      { "text": "/ɪˈfɛm(ə)ɹəl/", "audio": "" },
      { "text": "/əˈfɛm(ə)ɹəl/", "audio": "//ssl.gstatic.com/ephemeral.mp3" }
    ],
    "meanings": [
      {
        "partOfSpeech": "adjective",
        "definitions": [
          { "definition": " Lasting for a very short time. ", "example": "fashions are ephemeral", "synonyms": ["fleeting"] },
          { "definition": "(of a plant) having a very short life cycle.", "synonyms": [] }
        ],
        "synonyms": ["transient", "fleeting"]
      },
      { "partOfSpeech": "noun", "definitions": [{ "definition": "An ephemeral plant." }] }
    ]
  }
]"#;

#[test]
fn a_reply_becomes_senses_ipa_audio_and_synonyms() {
    let d = dictionaryapi_dev::from_json(REPLY, 7).unwrap().unwrap();
    assert_eq!(
        d.ipa.as_deref(),
        Some("/ɪˈfɛm(ə)ɹəl/"),
        "the first phonetic text that has one"
    );
    assert_eq!(
        d.audio.as_deref(),
        Some("https://ssl.gstatic.com/ephemeral.mp3")
    );
    assert_eq!(d.part_of_speech.as_deref(), Some("adjective"));
    assert_eq!(d.senses.len(), 3);
    assert_eq!(d.senses[0].definition, "Lasting for a very short time.");
    assert_eq!(
        d.senses[0].example.as_deref(),
        Some("fashions are ephemeral")
    );
    assert_eq!(d.senses[1].example, None);
    assert_eq!(d.synonyms, vec!["transient", "fleeting"]);
    assert_eq!(d.fetched_at, 7);
    let stored = serde_json::to_value(&d).unwrap();
    assert!(
        stored.get("partOfSpeech").is_some(),
        "stored in camelCase, as the UI reads it"
    );
}

#[test]
fn a_reply_without_definitions_is_no_entry_and_junk_is_an_error() {
    assert_eq!(dictionaryapi_dev::from_json("[]", 0).unwrap(), None);
    let err = dictionaryapi_dev::from_json(r#"{"title":"No Definitions Found"}"#, 0).unwrap_err();
    assert!(err.is(errors::WORD_LOOKUP));
}

#[test]
fn a_title_is_one_url_segment() {
    assert_eq!(segment("bite the bullet"), "bite%20the%20bullet");
    assert_eq!(segment("café/x"), "caf%C3%A9%2Fx");
    assert_eq!(segment("well-being"), "well-being");
}

const WIKTIONARY_REPLY: &str = r#"{
  "word": "laconic",
  "entries": [
    {
      "language": { "code": "en", "name": "English" },
      "partOfSpeech": "adjective",
      "pronunciations": [
        { "type": "enpr", "text": "lə-kŏnʹĭk", "tags": [] },
        { "type": "ipa", "text": "/ləˈkɒnɪk/", "tags": ["Received Pronunciation"] }
      ],
      "senses": [
        {
          "definition": " Using as few words as possible. ",
          "examples": ["Near-synonym: brief", "1821, A. Writer, A Book: a line", "His reply was laconic."],
          "quotes": [{ "text": "I grow laconick", "reference": "1738" }],
          "synonyms": ["terse"],
          "antonyms": ["verbose"],
          "subsenses": []
        },
        {
          "definition": "Of a person, laidback.",
          "examples": ["Synonyms: casual, relaxed"],
          "quotes": [{ "text": "A laconic key player.", "reference": "2019" }],
          "synonyms": [],
          "antonyms": [],
          "subsenses": []
        },
        { "definition": "Spartan.", "examples": [], "quotes": [] }
      ],
      "synonyms": ["concise", "terse"],
      "antonyms": ["wordy"]
    }
  ],
  "source": {
    "url": "https://en.wiktionary.org/wiki/laconic",
    "license": { "name": "CC BY-SA 4.0", "url": "https://creativecommons.org/licenses/by-sa/4.0/" }
  }
}"#;

#[test]
fn a_wiktionary_reply_becomes_senses_ipa_synonyms_antonyms_and_its_page() {
    let d = freedictionaryapi_com::from_json(WIKTIONARY_REPLY, 7)
        .unwrap()
        .unwrap();
    assert_eq!(d.source, Source::FreedictionaryapiCom);
    assert_eq!(
        d.source_url.as_deref(),
        Some("https://en.wiktionary.org/wiki/laconic")
    );
    assert_eq!(
        d.ipa.as_deref(),
        Some("/ləˈkɒnɪk/"),
        "the ipa one, not enpr"
    );
    assert_eq!(d.audio, None, "this source has no recordings");
    assert_eq!(d.part_of_speech.as_deref(), Some("adjective"));
    assert_eq!(d.senses[0].definition, "Using as few words as possible.");
    assert_eq!(
        d.senses[0].example.as_deref(),
        Some("His reply was laconic."),
        "not the thesaurus line, not the citation that starts with its year"
    );
    assert_eq!(
        d.senses[1].example.as_deref(),
        Some("A laconic key player."),
        "no usage example: the first quote"
    );
    assert_eq!(d.senses[2].example, None);
    assert_eq!(d.synonyms, vec!["concise", "terse"]);
    assert_eq!(d.antonyms, vec!["wordy", "verbose"]);
}

#[test]
fn a_wiktionary_reply_with_no_entries_is_no_entry_and_junk_is_an_error() {
    let empty =
        r#"{ "word": "zzzz", "entries": [], "source": { "url": "https://en.wiktionary.org" } }"#;
    assert_eq!(freedictionaryapi_com::from_json(empty, 0).unwrap(), None);
    let err = freedictionaryapi_com::from_json("[]", 0).unwrap_err();
    assert!(err.is(errors::WORD_LOOKUP));
}

#[test]
fn a_phrases_wiktionary_page_is_a_link_that_opens() {
    let phrase = r#"{ "entries": [{ "senses": [{ "definition": "To do the hard thing." }] }],
      "source": { "url": "https://en.wiktionary.org/wiki/bite the bullet" } }"#;
    let d = freedictionaryapi_com::from_json(phrase, 0)
        .unwrap()
        .unwrap();
    assert_eq!(
        d.source_url.as_deref(),
        Some("https://en.wiktionary.org/wiki/bite%20the%20bullet")
    );
}

#[test]
fn a_dictionary_stored_before_sources_reads_as_the_first_source_with_no_antonyms() {
    let old = r#"{ "ipa": null, "audio": null, "partOfSpeech": "noun", "senses": [], "synonyms": [], "fetchedAt": 3 }"#;
    let d: Dictionary = serde_json::from_str(old).unwrap();
    assert_eq!(d.source, Source::DictionaryapiDev);
    assert!(d.antonyms.is_empty());
    assert_eq!(
        serde_json::to_value(Source::FreedictionaryapiCom).unwrap(),
        "FREEDICTIONARYAPI_COM"
    );
}

#[test]
fn each_source_has_its_own_url() {
    assert_eq!(
        url_of(Source::DictionaryapiDev, "ephemeral"),
        "https://api.dictionaryapi.dev/api/v2/entries/en/ephemeral"
    );
    assert_eq!(
        url_of(Source::FreedictionaryapiCom, "bite%20the%20bullet"),
        "https://freedictionaryapi.com/api/v1/entries/en/bite%20the%20bullet"
    );
}
