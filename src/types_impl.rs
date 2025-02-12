use super::ifdformat::numarray_to_string;
use super::lowlevel::*;
use super::types::*;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::io;

/// Convert an IFD format code to the IfdFormat enumeration
pub fn ifdformat_new(n: u16) -> IfdFormat {
    match n {
        1 => IfdFormat::U8,
        2 => IfdFormat::Ascii,
        3 => IfdFormat::U16,
        4 => IfdFormat::U32,
        5 => IfdFormat::URational,
        6 => IfdFormat::I8,
        7 => IfdFormat::Undefined,
        8 => IfdFormat::I16,
        9 => IfdFormat::I32,
        10 => IfdFormat::IRational,
        11 => IfdFormat::F32,
        12 => IfdFormat::F64,
        _ => IfdFormat::Unknown,
    }
}

impl IfdEntry {
    /// Casts IFD entry data into an offset. Not very useful for the crate client.
    /// The call can't fail, but the caller must be sure that the IFD entry uses
    /// the IFD data area as an offset (i.e. when the tag is a Sub-IFD tag, or when
    /// there are more than 4 bytes of data and it would not fit within IFD).
    pub fn data_as_offset(&self) -> usize {
        read_u32(self.le, &(self.ifd_data[0..4])) as usize
    }

    /// Returns the size of an individual element (e.g. U8=1, U16=2...). Every
    /// IFD entry contains an array of elements, so this is NOT the size of the
    /// whole entry!
    pub fn size(&self) -> u8 {
        match self.format {
            IfdFormat::U8 => 1,
            IfdFormat::Ascii => 1,
            IfdFormat::U16 => 2,
            IfdFormat::U32 => 4,
            IfdFormat::URational => 8,
            IfdFormat::I8 => 1,
            IfdFormat::Undefined => 1,
            IfdFormat::I16 => 2,
            IfdFormat::I32 => 4,
            IfdFormat::IRational => 8,
            IfdFormat::F32 => 4,
            IfdFormat::F64 => 8,
            IfdFormat::Unknown => 1,
        }
    }

    /// Total length of the whole IFD entry (element count x element size)
    pub fn length(&self) -> usize {
        (self.size() as usize) * (self.count as usize)
    }

    /// Returns true if data is contained within the IFD structure, false when
    /// data can be found elsewhere in the image (and IFD structure contains the
    /// data offset, instead of data).
    pub fn in_ifd(&self) -> bool {
        self.length() <= 4
    }

    /// Copies data from IFD entry section reserved for data (up to 4 bytes), or
    /// from another part of the image file (when data wouldn't fit in IFD structure).
    /// In either case, the data member will contain the data of interest after
    /// this call.
    pub fn copy_data(&mut self, contents: &[u8]) -> bool {
        if self.in_ifd() {
            // the 4 bytes from IFD have all data
            self.data = self.ifd_data.clone();
            return true;
        }

        let offset = self.data_as_offset();
        if let Some(ext_data) = contents.get(offset..(offset + self.length())) {
            self.ext_data.clear();
            self.ext_data.extend(ext_data);
            self.data = self.ext_data.clone();
            return true;
        }
        false
    }
}

impl Error for ExifError {
    fn description(&self) -> &str {
        match *self {
            ExifError::IoError(ref e) => e.description(),
            ExifError::FileTypeUnknown => "File type unknown",
            ExifError::JpegWithoutExif(_) => "JPEG without EXIF section",
            ExifError::TiffTruncated => "TIFF truncated at start",
            ExifError::TiffBadPreamble(_) => "TIFF with bad preamble",
            ExifError::IfdTruncated => "TIFF IFD truncated",
            ExifError::ExifIfdTruncated(_) => "TIFF Exif IFD truncated",
            ExifError::ExifIfdEntryNotFound => "TIFF Exif IFD not found",
        }
    }
}

impl Display for ExifError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ExifError::IoError(ref e) => e.fmt(f),
            ExifError::FileTypeUnknown => write!(f, "File type unknown"),
            ExifError::JpegWithoutExif(ref s) => write!(f, "JPEG without EXIF section: {}", s),
            ExifError::TiffTruncated => write!(f, "TIFF truncated at start"),
            ExifError::TiffBadPreamble(ref s) => write!(f, "TIFF with bad preamble: {}", s),
            ExifError::IfdTruncated => write!(f, "TIFF IFD truncated"),
            ExifError::ExifIfdTruncated(ref s) => write!(f, "TIFF Exif IFD truncated: {}", s),
            ExifError::ExifIfdEntryNotFound => write!(f, "TIFF Exif IFD not found"),
        }
    }
}

impl From<io::Error> for ExifError {
    fn from(err: io::Error) -> ExifError {
        ExifError::IoError(err)
    }
}

impl fmt::Display for TagValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TagValue::Ascii(ref s) => write!(f, "{}", s),
            TagValue::U16(ref a) => write!(f, "{}", numarray_to_string(a)),
            TagValue::I16(ref a) => write!(f, "{}", numarray_to_string(a)),
            TagValue::U8(ref a) => write!(f, "{}", numarray_to_string(a)),
            TagValue::I8(ref a) => write!(f, "{}", numarray_to_string(a)),
            TagValue::U32(ref a) => write!(f, "{}", numarray_to_string(a)),
            TagValue::I32(ref a) => write!(f, "{}", numarray_to_string(a)),
            TagValue::F32(ref a) => write!(f, "{}", numarray_to_string(a)),
            TagValue::F64(ref a) => write!(f, "{}", numarray_to_string(a)),
            TagValue::URational(ref a) => write!(f, "{}", numarray_to_string(a)),
            TagValue::IRational(ref a) => write!(f, "{}", numarray_to_string(a)),
            TagValue::Undefined(ref a, _) => write!(f, "{}", numarray_to_string(a)),
            TagValue::Unknown(ref _a, _) => write!(f, "<unknown blob>"),
            TagValue::Invalid(ref _data, _le, _fmt, _cnt) => write!(f, "Invalid"),
        }
    }
}
