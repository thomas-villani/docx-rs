use docx_rs::read_docx;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;

const PACKAGE_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;

const DOCUMENT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#;

fn package(body: &str) -> Vec<u8> {
    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}</w:body></w:document>"#
    );
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, contents) in [
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", PACKAGE_RELS),
        ("word/_rels/document.xml.rels", DOCUMENT_RELS),
        ("word/document.xml", &document),
    ] {
        writer
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(contents.as_bytes()).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn read_body(body: &str) -> docx_rs::Docx {
    read_docx(&package(body)).unwrap()
}

// ECMA-376 allows a universal measure (a number with a unit such as `pt` or
// `in`) wherever a twips or half-point measurement is expected. Strict OOXML
// files use them for table widths, e.g. LibreOffice's tdf79272_strictDxa.docx.
#[test]
fn universal_measures_are_converted_to_twips_and_half_points() {
    let docx = read_body(
        r#"<w:tbl><w:tblPr><w:tblW w:w="122.40pt" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="1in"/></w:tblGrid><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl><w:p><w:pPr><w:spacing w:before="6pt" w:after="0.5in"/><w:ind w:left="1cm"/></w:pPr><w:r><w:rPr><w:sz w:val="20pt"/></w:rPr><w:t>text</w:t></w:r></w:p>"#,
    );
    let json: serde_json::Value = serde_json::from_str(&docx.json()).unwrap();
    let table = &json["document"]["children"][0]["data"];
    assert_eq!(table["property"]["width"]["width"], 2448);
    assert_eq!(table["grid"][0], 1440);
    let paragraph = &json["document"]["children"][1]["data"];
    assert_eq!(paragraph["property"]["lineSpacing"]["before"], 120);
    assert_eq!(paragraph["property"]["lineSpacing"]["after"], 720);
    assert_eq!(paragraph["property"]["indent"]["start"], 567);
    assert_eq!(paragraph["children"][0]["data"]["runProperty"]["sz"], 40);
}

// Invalid widths used to panic in read_width. Whether the read then fails or
// skips the property depends on the caller; either way it must not panic.
#[test]
fn invalid_widths_do_not_panic() {
    for body in [
        r#"<w:tbl><w:tblPr><w:tblW w:w="wide" w:type="dxa"/></w:tblPr></w:tbl>"#,
        r#"<w:tbl><w:tblGrid><w:gridCol w:w="wide"/></w:tblGrid></w:tbl>"#,
    ] {
        let _ = read_docx(&package(body));
    }
}
