use yrs::{Doc, Transact};

pub fn read(doc: &Doc) -> encre::Node {
    let fragment = doc.get_or_insert_xml_fragment(encre::yjs::FRAGMENT);
    encre::yjs::read(&doc.transact(), &fragment).unwrap()
}

pub fn type_into_first_paragraph(txn: &mut yrs::TransactionMut, fragment: &yrs::XmlFragmentRef, at: u32, text: &str) {
    use yrs::{GetString, Text, XmlFragment, XmlOut};
    let Some(XmlOut::Element(paragraph)) = fragment.get(txn, 0) else { panic!("no paragraph") };
    let Some(XmlOut::Text(run)) = paragraph.get(txn, 0) else { panic!("no text") };
    let at = at.min(run.get_string(txn).encode_utf16().count() as u32);
    run.insert(txn, at, text);
}
