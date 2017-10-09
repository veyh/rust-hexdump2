pub fn import(data: &str) -> Option<Vec<u8>> {
  let mut buffer = Vec::new();

  for line in data.split('\n') {
    let line = line.trim();
    let words = line.split(' ');
    let word_count = line.split(' ').count(); // TODO: improve

    for (index, word) in words.enumerate() {
      let len = word.len();

      if len == 0 {
        continue;
      }

      if let Ok(value) = u64::from_str_radix(word, 16) {
        if index == 0 && len > 2 {
          continue;
        }

        buffer.push(value as u8);
      }

      else if index == word_count - 1 {
        continue;
      }

      else {
        return None;
      }
    }
  }

  Some(buffer)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn import_with_bad_hex() {
    assert_eq!(None, import("xx aa bb"));
    assert_eq!(None, import("000000xx aa bb"));
  }

  #[test]
  fn import_with_just_bytes() {
    assert_eq!(
      Some(vec![0x00, 0x01, 0xfe, 0xff]),
      import("00 01 fe ff")
    );
  }

  #[test]
  fn import_ignores_extra_space() {
    assert_eq!(
      Some(vec![0x00, 0x01, 0xfe, 0xff]),
      import("00 01    fe ff")
    );
  }

  #[test]
  fn import_ignores_whitespace_and_line_breaks() {
    assert_eq!(
      Some(vec![0x00, 0x01, 0xfe, 0xff]),
      import("  00 01 \r\n  fe ff  \r")
    );
  }

  #[test]
  fn import_with_offsets_and_bytes() {
    assert_eq!(
      Some(vec![0x00, 0x01, 0xfe, 0xff]),
      import("0000 00 01 fe ff")
    );

    assert_eq!(
      Some(vec![0x00, 0x01, 0xfe, 0xff]),
      import("00000000 00 01 fe ff")
    );

    assert_eq!(
      Some(vec![0x00, 0x01, 0xfe, 0xff]),
      import("0000000000000000 00 01 fe ff")
    );
  }

  #[test]
  fn import_with_bytes_and_strings() {
    assert_eq!(
      Some(vec![0x61, 0x62, 0x63, 0x64, 0x00, 0x19, 0x7f, 0x10]),
      import("61 62 63 64 00 19 7f 10 abcd....")
    );
  }

  #[test]
  fn import_with_offsets_bytes_and_strings() {
    assert_eq!(
      Some(vec![0x61, 0x62, 0x63, 0x64, 0x00, 0x19, 0x7f, 0x10]),
      import("0000 61 62 63 64 00 19 7f 10 abcd....")
    );
  }
}
