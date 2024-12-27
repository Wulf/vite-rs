use lazy_static::lazy_static;
use std::path::PathBuf;

use regex::Regex;
lazy_static! {
    // regex for <script/> statements which specify "data-bundle" attribute
    static ref SCRIPT_TAG_REGEX: Regex = Regex::new(r#"<\s*script\s+.*?data-bundle.*?\s*>"#).unwrap();
}

lazy_static! {
    // regex for src attribute in <script/> tags
    static ref SRC_ATTRIBUTE_REGEX: Regex = Regex::new(r#"<\s*script\s+.*?src=['"](.*?)['"].*?\s*>"#).unwrap();
}

/// This method may return paths that don't exist since it's only looking at the src attribute in <script/> tags
pub fn get_script_tag_sources(
    html_file_path: &PathBuf,
    html_file_contents: &Vec<u8>,
) -> Vec<PathBuf> {
    let mut files_to_bundle = Vec::new();

    for cap in SCRIPT_TAG_REGEX.captures_iter(std::str::from_utf8(&html_file_contents).unwrap()) {
        let script_tag = cap.get(0).unwrap();

        let src_attribute = SRC_ATTRIBUTE_REGEX
            .captures(script_tag.as_str())
            .unwrap()
            .get(1)
            .unwrap()
            .as_str();

        let mut script_path = html_file_path.clone();
        script_path.pop(); // remove the file name
                           // script_path.push(format!("{src_attribute}.bundle.js"));
        script_path.push(src_attribute);

        files_to_bundle.push(script_path);
    }

    files_to_bundle
}

// TODO: accept fn to mutate src attribute instead of hardcoding /.cook/{src_attribute}/bundle.js
pub fn update_src_attributes(html_data: &mut Vec<u8>) {
    let data = html_data;
    let mut offset: isize = 0;

    for cap in SCRIPT_TAG_REGEX.captures_iter(std::str::from_utf8(&data.clone()).unwrap()) {
        let script_tag = cap.get(0).unwrap();
        let script_tag_position = script_tag.range();
        let script_tag_txt = script_tag.as_str();
        let src_attribute = SRC_ATTRIBUTE_REGEX.captures(script_tag_txt).unwrap().get(1);
        let src_attr_position = src_attribute.unwrap().range();
        let (start, end) = (
            src_attr_position.start + script_tag_position.start,
            src_attr_position.end + script_tag_position.start,
        );
        let src_attribute_txt = src_attribute.unwrap().as_str();
        let new_src_attribute_txt = format!("/.cook/{}/bundle.js", src_attribute_txt);

        // realistically, we won't hit the bounds of this usize->isize conversion, but it could be a potential bug
        if start >= isize::MAX as usize
            || end >= isize::MAX as usize
            || start + offset as usize >= isize::MAX as usize
            || end + offset as usize >= isize::MAX as usize
        {
            panic!("hit bounds of usize->isize conversion; file too large to process (case: a)");
        }

        let start_isize = start as isize + offset;
        let end_isize = end as isize + offset;

        if start_isize < 0 || end_isize < 0 || start_isize as usize >= data.len() {
            println!(
                "start_isize: {}, end_isize: {}, data.len(): {}",
                start_isize,
                end_isize,
                data.len()
            );
            panic!("hit bounds of usize->isize conversion; file too large to process (case: b)");
        }

        data.splice(
            start_isize as usize..end_isize as usize,
            new_src_attribute_txt.bytes(),
        );

        offset += new_src_attribute_txt.len() as isize - src_attribute_txt.len() as isize;
    }
}

#[test]
fn test_update_src_attributes() {
    let u8_to_string = |data: Vec<u8>| -> String { String::from_utf8(data.clone()).unwrap() };

    // single script tag test
    let mut data = b"<html><script src=\"file.js\" data-bundle></script></html>".to_vec();
    update_src_attributes(&mut data);
    assert_eq!(
        u8_to_string(data),
        u8_to_string(
            b"<html><script src=\"/.cook/file.js/bundle.js\" data-bundle></script></html>".to_vec()
        )
    );

    // multiple script tags test
    let mut data = b"<html><script src=\"file.js\" data-bundle></script><script src=\"file2.js\" data-bundle></script></html>".to_vec();
    update_src_attributes(&mut data);
    assert_eq!(
        u8_to_string(data),
        u8_to_string(b"<html><script src=\"/.cook/file.js/bundle.js\" data-bundle></script><script src=\"/.cook/file2.js/bundle.js\" data-bundle></script></html>".to_vec())
    );

    // no data-bundle test
    let mut data = b"<html><script src=\"file.js\"></script></html>".to_vec();
    update_src_attributes(&mut data);
    assert_eq!(
        u8_to_string(data),
        u8_to_string(b"<html><script src=\"file.js/bundle.js\"></script></html>".to_vec())
    );
}
