use std::io::Read;
use std::str::FromStr;

use super::*;

impl FromXML for Footnotes {
    fn from_xml<R: Read>(reader: R) -> Result<Self, ReaderError> {
        let mut r = EventReader::new(reader);
        let mut footnotes: Vec<Footnote> = vec![];
        loop {
            let e = r.next_event();
            match e {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => {
                    let e = XMLElement::from_str(&name.local_name)
                        .expect("should convert to XMLElement");
                    if let XMLElement::Footnote = e {
                        let footnote = read_footnote(&mut r)?;
                        // Separator and continuation notes (w:type) are not referenced
                        // from the document, and their ids may be negative.
                        if read(&attributes, "type").is_none() {
                            if let Some(id) =
                                read(&attributes, "id").and_then(|id| usize::from_str(&id).ok())
                            {
                                footnotes.push(Footnote {
                                    id,
                                    content: footnote,
                                });
                            }
                        }
                    }
                }
                Ok(XmlEvent::EndElement { name, .. }) => {
                    let e = XMLElement::from_str(&name.local_name).unwrap();
                    if e == XMLElement::Footnotes {
                        return Ok(Footnotes { footnotes });
                    }
                }
                Ok(XmlEvent::EndDocument) => {
                    return Ok(Footnotes { footnotes });
                }
                Err(_) => return Err(ReaderError::XMLReadError),
                _ => {}
            }
        }
    }
}

// Paragraphs nested in tables or content controls are collected as note
// paragraphs, since a footnote holds paragraphs only.
fn read_footnote<R: Read>(r: &mut EventReader<R>) -> Result<Vec<Paragraph>, ReaderError> {
    let mut paragraphs = vec![];
    loop {
        let e = r.next_event();
        match e {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                let e =
                    XMLElement::from_str(&name.local_name).expect("should convert to XMLElement");
                if let XMLElement::Paragraph = e {
                    paragraphs.push(Paragraph::read(r, &attributes)?);
                }
            }
            Ok(XmlEvent::EndElement { name, .. }) => {
                let e = XMLElement::from_str(&name.local_name).unwrap();
                if e == XMLElement::Footnote {
                    return Ok(paragraphs);
                }
            }
            Err(_) => return Err(ReaderError::XMLReadError),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(test)]
    use pretty_assertions::assert_eq;

    #[test]
    fn test_read_footnotes() {
        let xml = r#"<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote>
  <w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>
  <w:footnote w:id="1">
    <w:p><w:r><w:footnoteRef/></w:r><w:r><w:t>First note</w:t></w:r></w:p>
    <w:tbl><w:tr><w:tc><w:p><w:r><w:t>In a table</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
  </w:footnote>
</w:footnotes>"#;
        let footnotes = Footnotes::from_xml(xml.as_bytes()).unwrap();
        assert_eq!(footnotes.footnotes.len(), 1);
        let footnote = &footnotes.footnotes[0];
        assert_eq!(footnote.id, 1);
        let text: Vec<String> = footnote.content.iter().map(|p| p.raw_text()).collect();
        assert_eq!(text, vec!["First note", "In a table"]);
    }
}
