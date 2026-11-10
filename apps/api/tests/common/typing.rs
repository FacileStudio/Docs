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

pub fn callouts(depth: usize) -> encre::Node {
    let mut node: encre::Node =
        serde_json::from_value(serde_json::json!({"type": "paragraph", "content": [{"type": "text", "text": "deep"}]}))
            .unwrap();
    for _ in 0..depth {
        let callout = serde_json::json!({"type": "callout", "attrs": {"tone": "info", "title": ""}, "content": [node]});
        node = serde_json::from_value(callout).unwrap();
    }
    let mut doc: encre::Node = serde_json::from_value(serde_json::json!({"type": "doc", "content": [node]})).unwrap();
    encre::assign_ids(&mut doc);
    doc
}

pub fn nest_one_more_callout(txn: &mut yrs::TransactionMut, fragment: &yrs::XmlFragmentRef) {
    use yrs::{Xml, XmlElementPrelim, XmlFragment, XmlOut};
    let Some(XmlOut::Element(mut innermost)) = fragment.get(txn, 0) else { panic!("no callout") };
    while let Some(XmlOut::Element(child)) =
        innermost.get(txn, 0).filter(|c| matches!(c, XmlOut::Element(e) if e.tag().as_ref() == "callout"))
    {
        innermost = child;
    }
    let callout = innermost.insert(txn, 0, XmlElementPrelim::empty("callout"));
    callout.insert_attribute(txn, "id", "0000000000");
    callout.insert_attribute(txn, "tone", "info");
    callout.insert_attribute(txn, "title", "");
    let paragraph = callout.insert(txn, 0, XmlElementPrelim::empty("paragraph"));
    paragraph.insert_attribute(txn, "id", "0000000001");
}
