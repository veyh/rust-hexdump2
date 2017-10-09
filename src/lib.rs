/// Reads a hexdump string into bytes.
///
/// # Examples
///
/// With only a string of bytes.
///
/// ```
/// assert_eq!(
///   Some(vec![0x00, 0x01, 0xfe, 0xff]),
///   hexdump2::import("00 01 fe ff")
/// );
/// ```
///
/// With offsets.
///
/// ```
/// assert_eq!(
///   Some(vec![0x00, 0x01, 0xfe, 0xff]),
///   hexdump2::import("0000 00 01 fe ff")
/// );
/// ```
///
/// With ascii.
///
/// ```
/// assert_eq!(
///   Some(vec![0x61, 0x62, 0x63, 0x64, 0x00, 0x19, 0x7f, 0x10]),
///   hexdump2::import("61 62 63 64 00 19 7f 10 abcd....")
/// );
/// ```
///
/// With offsets and ascii.
///
/// ```
/// assert_eq!(
///   Some(vec![0x61, 0x62, 0x63, 0x64, 0x00, 0x19, 0x7f, 0x10]),
///   hexdump2::import("0000 61 62 63 64 00 19 7f 10 abcd....")
/// );
/// ```
///
/// Works with multiple lines, too. Minimum number of values on a line is 3.
///
/// ```
/// assert_eq!(
///   Some(vec![0x61, 0x62, 0x63, 0x64, 0x65]),
///   hexdump2::import(r#"0000 61 62 63 abc
///                       0003 64 65    de"#)
/// );
/// ```
pub fn import(data: &str) -> Option<Vec<u8>> {
  let mut buffer = Vec::new();

  for line in data.split('\n') {
    let line = line.trim();
    let words = line.split(' ');
    let word_count = line.split(' ').count(); // TODO: improve
    let mut had_padding = false;

    for (index, word) in words.enumerate() {
      let len = word.len();

      // extra space
      if len == 0 {
        had_padding = true;
        continue;
      }

      // offset
      else if index == 0 && len > 2 {
        continue;
      }

      else if len == 2 {
        if index == word_count - 1
        && had_padding {
          continue;
        }

        else if let Ok(value) = u64::from_str_radix(word, 16) {
          buffer.push(value as u8);
          had_padding = false;
        }

        else {
          return None;
        }
      }
    }
  }

  Some(buffer)
}

pub struct ExportOptions {
  pub per_line: usize,
  pub with_offsets: bool,
  pub with_ascii: bool,
}


/// Exports a slice of bytes into a hexdump string.
pub fn export(
  values: &[u8],
  options: ExportOptions
) -> Result<String, ExportError> {
  let mut res = String::new();
  export_to(&mut res, values, options)?;
  Ok(res)
}

#[derive(Debug, PartialEq)]
pub enum ExportError {
  Fmt(::std::fmt::Error),
  BadOptions,
}

/// Exports a slice of bytes into a writable.
pub fn export_to<T: ::std::fmt::Write>(
  target: &mut T,
  values: &[u8],
  options: ExportOptions
) -> Result<(), ExportError> {
  let total_value_count = values.len();
  let mut line_value_count = 0;
  let mut ascii = String::new();

  for (index, value) in values.iter().enumerate() {
    if options.with_offsets
    && index % options.per_line == 0 {
      write_offset(target, index, total_value_count).unwrap();
    }

    target.write_str(&format!("{:02X}", *value)).unwrap();
    line_value_count += 1;

    if options.with_ascii {
      push_ascii(&mut ascii, *value);
    }

    let is_last_value = index == total_value_count - 1;

    if is_last_value {
      if options.with_ascii {
        write_ascii(target, &ascii, line_value_count,
                    options.per_line).unwrap();
      }

      continue;
    }

    let is_last_value_for_this_line = line_value_count == options.per_line;

    if is_last_value_for_this_line {
      if options.with_ascii {
        write_ascii(target, &ascii, line_value_count,
                    options.per_line).unwrap();
        ascii.clear();
      }

      target.write_char('\n').unwrap();
      line_value_count = 0;
      continue;
    }

    if line_value_count < options.per_line {
      target.write_char(' ').unwrap();
    }
  }

  Ok(())
}

fn write_offset<T: ::std::fmt::Write>(
  target: &mut T,
  index: usize,
  total_value_count: usize
) -> Result<(), ::std::fmt::Error> {
  if total_value_count <= 0xffff {
    target.write_str(&format!("{:04X} ", index))
  }

  else if total_value_count <= 0xffffffff {
    target.write_str(&format!("{:08X} ", index))
  }

  else {
    target.write_str(&format!("{:16X} ", index))
  }
}

fn push_ascii(ascii: &mut String, value: u8) {
  if value >= 0x20 && value <= 0x7e {
    ascii.push(value as char);
  }

  else {
    ascii.push('.');
  }
}

fn write_ascii<T: ::std::fmt::Write>(
  target: &mut T,
  ascii: &String,
  count: usize,
  per_line: usize,
) -> Result<(), ::std::fmt::Error> {
  let missing_value_count = per_line - count;

  for _ in 0..missing_value_count {
    target.write_str("   ")?;
  }
  target.write_char(' ')?;
  target.write_str(&ascii)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn import_with_bad_hex() {
    assert_eq!(None, import("xx aa bb"));
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
  fn exports_bytes() {
    let mut actual = String::new();
    assert_eq!(Ok(()), export_to(&mut actual, &[0, 1, 2, 3], ExportOptions {
      with_ascii: false,
      with_offsets: false,
      per_line: 16,
    }));
    assert_eq!("00 01 02 03", &actual);
  }

  #[test]
  fn export_bytes_on_multiple_lines() {
    let mut actual = String::new();
    assert_eq!(Ok(()), export_to(&mut actual, &[0, 1, 2, 3], ExportOptions {
      with_ascii: false,
      with_offsets: false,
      per_line: 2,
    }));
    assert_eq!("00 01\n02 03", &actual);
  }

  #[test]
  fn exports_offsets_and_bytes() {
    let mut actual = String::new();
    assert_eq!(Ok(()), export_to(&mut actual, &[0, 1, 2, 3], ExportOptions {
      with_ascii: false,
      with_offsets: true,
      per_line: 2,
    }));
    assert_eq!("0000 00 01\n0002 02 03", &actual);
  }

  #[test]
  fn exports_offsets_bytes_and_ascii() {
    let mut actual = String::new();
    assert_eq!(Ok(()), export_to(&mut actual, &[
      0x61, 0x62, 0x63, 0x64,
      0x65, 0x00, 0x19, 0x7f
    ], ExportOptions {
      with_ascii: true,
      with_offsets: true,
      per_line: 4,
    }));
    assert_eq!("0000 61 62 63 64 abcd\n0004 65 00 19 7F e...", &actual);
  }

  #[test]
  fn exports_offsets_bytes_and_ascii_with_partial_last_line() {
    let mut actual = String::new();
    assert_eq!(Ok(()), export_to(&mut actual, &[
      0x61, 0x62, 0x63, 0x64,
      0x65, 0x00,
    ], ExportOptions {
      with_ascii: true,
      with_offsets: true,
      per_line: 4,
    }));
    assert_eq!("0000 61 62 63 64 abcd\n0004 65 00       e.", &actual);
  }

  #[test]
  fn exports_to_string() {
    assert_eq!(
      Ok(String::from("00 01 02 03")),
      export(&[0, 1, 2, 3], ExportOptions {
        with_ascii: false,
        with_offsets: false,
        per_line: 4,
      })
    );
  }
}
