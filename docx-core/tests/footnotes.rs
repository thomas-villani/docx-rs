use docx_rs::read_docx;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/footnotes.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml"/></Types>"#;

const PACKAGE_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;

const DOCUMENT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes" Target="footnotes.xml"/></Relationships>"#;

// One reference in a body paragraph, one inside a table cell.
const DOCUMENT: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
<w:p><w:r><w:t>Body text</w:t></w:r><w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteReference w:id="1"/></w:r></w:p>
<w:tbl><w:tr><w:tc><w:p><w:r><w:t>Cell text</w:t></w:r><w:r><w:footnoteReference w:id="2"/></w:r></w:p></w:tc></w:tr></w:tbl>
</w:body></w:document>"#;

const FOOTNOTES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote>
<w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>
<w:footnote w:id="1"><w:p><w:r><w:footnoteRef/></w:r><w:r><w:t>First note</w:t></w:r></w:p></w:footnote>
<w:footnote w:id="2"><w:p><w:r><w:footnoteRef/></w:r><w:r><w:t>Cell note</w:t></w:r></w:p></w:footnote>
</w:footnotes>"#;

fn package(parts: &[(&str, &str)]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, contents) in parts {
        writer
            .start_file(*name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(contents.as_bytes()).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn document_with_footnotes() -> Vec<u8> {
    package(&[
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", PACKAGE_RELS),
        ("word/_rels/document.xml.rels", DOCUMENT_RELS),
        ("word/document.xml", DOCUMENT),
        ("word/footnotes.xml", FOOTNOTES),
    ])
}

#[test]
fn footnotes_are_read() {
    let json = read_docx(&document_with_footnotes()).unwrap().json();
    assert!(json.contains(r#""type": "footnoteReference""#));
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let footnotes = value["footnotes"]["footnotes"].as_array().unwrap();
    // Separator and continuation notes are not document content.
    let ids: Vec<u64> = footnotes
        .iter()
        .map(|f| f["id"].as_u64().unwrap())
        .collect();
    assert_eq!(ids, vec![1, 2]);
    assert!(footnotes[0].to_string().contains("First note"));
    assert!(footnotes[1].to_string().contains("Cell note"));
}

#[test]
fn read_footnotes_are_written_once() {
    let xml = read_docx(&document_with_footnotes()).unwrap().build();
    let footnotes = String::from_utf8(xml.footnotes).unwrap();
    assert_eq!(footnotes.matches("First note").count(), 1);
    assert_eq!(footnotes.matches("Cell note").count(), 1);
    let document = String::from_utf8(xml.document).unwrap();
    assert!(document.contains(r#"<w:footnoteReference w:id="1" />"#));
    assert!(document.contains(r#"<w:footnoteReference w:id="2" />"#));
    let rels = String::from_utf8(xml.document_rels).unwrap();
    assert!(rels.contains("relationships/footnotes"));
}

#[test]
fn missing_footnotes_part_is_ignored() {
    let bytes = package(&[
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", PACKAGE_RELS),
        ("word/_rels/document.xml.rels", DOCUMENT_RELS),
        ("word/document.xml", DOCUMENT),
    ]);
    let json = read_docx(&bytes).unwrap().json();
    assert!(json.contains("Body text"));
    assert!(json.contains(r#""type": "footnoteReference""#));
}
