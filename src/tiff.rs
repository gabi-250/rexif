use super::exif::*;
use super::exifpost::*;
use super::ifdformat::*;
use super::lowlevel::*;
use super::types::*;
use super::types_impl::*;

type InExifResult = Result<(), ExifError>;

/// Parse of raw IFD entry into EXIF data, if it is of a known type, and returns
/// an ExifEntry object. If the tag is unknown, the enumeration is set to UnknownToMe,
/// but the raw information of tag is still available in the ifd member.
pub fn parse_exif_entry(f: &IfdEntry, warnings: &mut Vec<String>) -> ExifEntry {
    let value = tag_value_new(f);

    let mut e = ExifEntry {
        namespace: f.namespace,
        ifd: f.clone(),
        tag: ExifTag::UnknownToMe,
        value: value.clone(),
        unit: "Unknown".to_string(),
        value_more_readable: format!("{}", value),
    };

    let (tag, unit, format, min_count, max_count, more_readable) = tag_to_exif(f.tag);

    if tag == ExifTag::UnknownToMe {
        // Unknown EXIF tag type
        return e;
    }

    // Internal assert:
    // 1) tag must match enum
    // 2) all types except Ascii, Undefined, Unknown must have definite length
    // 3) Str type must not have a definite length
    if (((tag as u32) & 0xffff) as u16) != f.tag
        || (min_count == -1
            && (format != IfdFormat::Ascii
                && format != IfdFormat::Undefined
                && format != IfdFormat::Unknown))
        || (min_count != -1 && format == IfdFormat::Ascii)
    {
        panic!("Internal error {:x}", f.tag);
    }

    if format != f.format {
        warnings.push(format!(
            "EXIF tag {:x} {} ({}), expected format {} ({:?}), found {} ({:?})",
            f.tag, f.tag, tag, format as u8, format, f.format as u8, f.format
        ));
        return e;
    }

    if min_count != -1 && ((f.count as i32) < min_count || (f.count as i32) > max_count) {
        warnings.push(format!(
            "EXIF tag {:x} {} ({:?}), format {}, expected count {}..{} found {}",
            f.tag, f.tag, tag, format as u8, min_count, max_count, f.count
        ));
        return e;
    }

    e.tag = tag;
    e.unit = unit.to_string();
    e.value_more_readable = more_readable(&e.value);

    e
}

/// Superficial parse of IFD that can't fail
pub fn parse_ifd(
    subifd: bool,
    le: bool,
    count: u16,
    contents: &[u8],
) -> Option<(Vec<IfdEntry>, usize)> {
    let mut entries: Vec<IfdEntry> = Vec::new();

    for i in 0..count {
        let mut offset = (i as usize) * 12;
        let tag = read_u16(le, &contents.get(offset..offset + 2)?);
        offset += 2;
        let format = read_u16(le, &contents.get(offset..offset + 2)?);
        offset += 2;
        let count = read_u32(le, &contents.get(offset..offset + 4)?);
        offset += 4;
        let data = &contents.get(offset..offset + 4)?;
        let data = data.to_vec();

        let entry = IfdEntry {
            namespace: Namespace::Standard,
            tag,
            format: ifdformat_new(format),
            count,
            ifd_data: data,
            le,
            ext_data: Vec::new(),
            data: Vec::new(),
        };
        entries.push(entry);
    }

    let next_ifd = if subifd {
        0
    } else {
        read_u32(le, &contents[count as usize * 12..]) as usize
    };

    Some((entries, next_ifd))
}

/// Deep parse of IFD that grabs EXIF data from IFD0, SubIFD and GPS IFD
fn parse_exif_ifd(
    le: bool,
    contents: &[u8],
    ioffset: usize,
    exif_entries: &mut Vec<ExifEntry>,
    warnings: &mut Vec<String>,
) -> InExifResult {
    let mut offset = ioffset;

    if contents.len() < (offset + 2) {
        return Err(ExifError::ExifIfdTruncated(
            "Truncated at dir entry count".to_string(),
        ));
    }

    let count = read_u16(
        le,
        &contents
            .get(offset..offset + 2)
            .ok_or(ExifError::IfdTruncated)?,
    );
    let ifd_length = (count as usize) * 12;
    offset += 2;

    if contents.len() < (offset + ifd_length) {
        return Err(ExifError::ExifIfdTruncated(
            "Truncated at dir listing".to_string(),
        ));
    }

    let ifd_content = &contents
        .get(offset..offset + ifd_length)
        .ok_or(ExifError::IfdTruncated)?;
    let (mut ifd, _) = parse_ifd(true, le, count, ifd_content).ok_or(ExifError::IfdTruncated)?;

    for entry in &mut ifd {
        if !entry.copy_data(contents) {
            // data is probably beyond EOF
            continue;
        }
        let exif_entry = parse_exif_entry(entry, warnings);
        exif_entries.push(exif_entry);
    }

    Ok(())
}

/// Parses IFD0 and looks for SubIFD or GPS IFD within IFD0
pub fn parse_ifds(
    le: bool,
    ifd0_offset: usize,
    contents: &[u8],
    warnings: &mut Vec<String>,
) -> ExifEntryResult {
    let mut offset = ifd0_offset;
    let mut exif_entries: Vec<ExifEntry> = Vec::new();

    // fills exif_entries with data from IFD0

    match parse_exif_ifd(le, contents, offset, &mut exif_entries, warnings) {
        Ok(_) => true,
        Err(e) => return Err(e),
    };

    // at this point we knot that IFD0 is good
    // looks for SubIFD (EXIF)

    let count = read_u16(
        le,
        &contents
            .get(offset..offset + 2)
            .ok_or(ExifError::IfdTruncated)?,
    );
    let ifd_length = (count as usize) * 12 + 4;
    offset += 2;

    let ifd_content = &contents
        .get(offset..offset + ifd_length)
        .ok_or(ExifError::IfdTruncated)?;
    let (ifd, _) = parse_ifd(false, le, count, ifd_content).ok_or(ExifError::IfdTruncated)?;

    for entry in &ifd {
        if entry.tag != (((ExifTag::ExifOffset as u32) & 0xffff) as u16)
            && entry.tag != (((ExifTag::GPSOffset as u32) & 0xffff) as u16)
        {
            continue;
        }

        let exif_offset = entry.data_as_offset();

        if contents.len() < exif_offset {
            return Err(ExifError::ExifIfdTruncated(
                "Exif SubIFD goes past EOF".to_string(),
            ));
        }

        match parse_exif_ifd(le, contents, exif_offset, &mut exif_entries, warnings) {
            Ok(_) => true,
            Err(e) => return Err(e),
        };
    }

    // I didn't want to make the copy, but how to pass a vector that is
    // being iterated onto?
    let exif_entries_copy = exif_entries.clone();

    for entry in &mut exif_entries {
        exif_postprocessing(entry, &exif_entries_copy);
    }

    Ok(exif_entries)
}

/// Parse a TIFF image, or embedded TIFF in JPEG, in order to get IFDs and then the EXIF data
pub fn parse_tiff(contents: &[u8], warnings: &mut Vec<String>) -> ExifEntryResult {
    let mut le = false;

    if contents.len() < 8 {
        return Err(ExifError::TiffTruncated);
    } else if contents[0] == b'I' && contents[1] == b'I' && contents[2] == 42 && contents[3] == 0 {
        /* TIFF little-endian */
        le = true;
    } else if contents[0] == b'M' && contents[1] == b'M' && contents[2] == 0 && contents[3] == 42 {
        /* TIFF big-endian */
    } else {
        let err = format!(
            "Preamble is {:x} {:x} {:x} {:x}",
            contents[0], contents[1], contents[2], contents[3]
        );
        return Err(ExifError::TiffBadPreamble(err));
    }

    let offset = read_u32(le, &contents[4..8]) as usize;

    parse_ifds(le, offset, &contents, warnings)
}
