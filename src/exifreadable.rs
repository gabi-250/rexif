use super::types::*;
use super::ifdformat::*;
use super::lowlevel::read_u16_array;

static INV: &'static str = "Invalid data for this tag";

/// No-op for readable value tag function. Should not be used by any EXIF tag descriptor,
/// except for the catch-all match that handles unknown tags
pub fn nop(_: &TagValue, s: &String) -> String
{
	return s.clone();
}

/// No-op for readable value tag function. Used for ASCII string tags, or when the
/// default readable representation of value is pretty enough.
pub fn strpass(_: &TagValue, s: &String) -> String
{
	return s.clone();
}

pub fn orientation(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::U16(ref v) => {
			let n = v[0];
			match n {
				1 => "Straight",
				3 => "Upside down",
				6 => "Rotated to left",
				8 => "Rotated to right",
				9 => "Undefined",
				_ => return format!("Unknown ({})", n),
			}
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn rational_value(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::URational(ref v) => {
			format!("{}", v[0].value())
		},
		&TagValue::IRational(ref v) => {
			format!("{}", v[0].value())
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn rational_values(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::URational(ref v) => {
			let ve: Vec<f64> = v.iter().map(|&x| x.value()).collect();
			numarray_to_string(&ve)
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn resolution_unit(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::U16(ref v) => {
			let n = v[0];
			match n {
				1 => "Unitless",
				2 => "in",
				3 => "cm",
				_ => return format!("Unknown ({})", n),
			}
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn exposure_time(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::URational(ref v) => {
			format!("{} s", v[0])
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn f_number(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::URational(ref v) => {
			format!("f/{:.1}", v[0].value())
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn exposure_program(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::U16(ref v) => {
			let n = v[0];
			match n {
				1 => "Manual control",
				2 => "Program control",
				3 => "Aperture priority",
				4 => "Shutter priority",
				5 => "Program creative (slow program)",
				6 => "Program creative (high-speed program)",
				7 => "Portrait mode",
				8 => "Landscape mode",
				_ => return format!("Unknown ({})", n),
			}
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn focal_length(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::URational(ref v) => {
			format!("{} mm", v[0].value())
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn focal_length_35(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::U16(ref v) => {
			format!("{} mm", v[0])
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn meters(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::URational(ref v) => {
			format!("{:.1} m", v[0].value())
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn iso_speeds(e: &TagValue, _: &String) -> String
{
	let s = match e {
	&TagValue::U16(ref v) => {
		if v.len() == 1 {
			format!("ISO {}", v[0])
		} else if v.len() == 2 {
			format!("ISO {} latitude {}", v[0], v[1])
		} else {
			format!("Unknown ({})", numarray_to_string(&v))
		}
	},
	_ => panic!(INV),
	};

	return s.to_string();
}

pub fn dms(e: &TagValue, _: &String) -> String
{
	let s = match e {
	&TagValue::URational(ref v) => {
		let deg = v[0];
		let min = v[1];
		let sec = v[2];
		if deg.denominator == 1 && min.denominator == 1 {
			format!("{}°{}'{:.2}\"", deg.value(), min.value(), sec.value())
		} else if deg.denominator == 1 {
			format!("{}°{:.4}'", deg.value(), min.value() + sec.value() / 60.0)
		} else {
			// untypical format
			format!("{:.7}°", deg.value() + min.value() / 60.0 + sec.value() / 3600.0)
		}
	},
	_ => panic!(INV),
	};

	return s.to_string();
}

pub fn gps_alt_ref(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::U8(ref v) => {
			let n = v[0];
			match n {
				0 => "Above sea level",
				1 => "Below sea level",
				_ => return format!("Unknown, assumed below sea level ({})", n),
			}
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn gpsdestdistanceref(e: &TagValue, _: &String) -> String
{
	let s = match e {
	&TagValue::Ascii(ref v) => {
		if v == "N" {
			"kn"
		} else if v == "K" {
			"km"
		} else if v == "M" {
			"mi"
		} else {
			return format!("Unknown ({})", v)
		}
	},
	_ => panic!(INV),
	};

	return s.to_string();
}

pub fn gpsdestdistance(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::URational(ref v) => {
			format!("{:.3}", v[0].value())
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn gpsspeedref(e: &TagValue, _: &String) -> String
{
	let s = match e {
	&TagValue::Ascii(ref v) => {
		if v == "N" {
			"kn"
		} else if v == "K" {
			"km/h"
		} else if v == "M" {
			"mi/h"
		} else {
			return format!("Unknown ({})", v)
		}
	},
	_ => panic!(INV),
	};

	return s.to_string();
}

pub fn gpsspeed(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::URational(ref v) => {
			format!("{:.1}", v[0].value())
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn gpsbearingref(e: &TagValue, _: &String) -> String
{
	let s = match e {
	&TagValue::Ascii(ref v) => {
		if v == "T" {
			"True bearing"
		} else if v == "M" {
			"Magnetic bearing"
		} else {
			return format!("Unknown ({})", v)
		}
	},
	_ => panic!(INV),
	};

	return s.to_string();
}

pub fn gpsbearing(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::URational(ref v) => {
			format!("{:.2}°", v[0].value())
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn gpstimestamp(e: &TagValue, _: &String) -> String
{
	let s = match e {
	&TagValue::URational(ref v) => {
		let hour = v[0];
		let min = v[1];
		let sec = v[2];
		format!("{:02.0}:{:02.0}:{:02.1} UTC", hour.value(), min.value(), sec.value())
	},
	_ => panic!(INV),
	};

	return s.to_string();
}

pub fn gpsdiff(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::U16(ref v) => {
			let n = v[0];
			match n {
				0 => "Measurement without differential correction",
				1 => "Differential correction applied",
				_ => return format!("Unknown ({})", n),
			}
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn gpsstatus(e: &TagValue, _: &String) -> String
{
	let s = match e {
	&TagValue::Ascii(ref v) => {
		if v == "A" {
			"Measurement in progress"
		} else if v == "V" {
			"Measurement is interoperability"
		} else {
			return format!("Unknown ({})", v)
		}
	},
	_ => panic!(INV),
	};

	return s.to_string();
}

pub fn gpsmeasuremode(e: &TagValue, _: &String) -> String
{
	let s = match e {
	&TagValue::Ascii(ref v) => {
		if v == "2" {
			"2-dimension"
		} else if v == "3" {
			"3-dimension"
		} else {
			return format!("Unknown ({})", v)
		}
	},
	_ => panic!(INV),
	};

	return s.to_string();
}

pub fn undefined_as_ascii(e: &TagValue, _: &String) -> String
{
	let s = match e {
	&TagValue::Undefined(ref v, _) => {
		String::from_utf8_lossy(&v[..])
	},
	_ => panic!(INV),
	};

	return s.to_string();
}

pub fn undefined_as_u8(e: &TagValue, _: &String) -> String
{
	let s = match e {
	&TagValue::Undefined(ref v, _) => {
		numarray_to_string(v)
	},
	_ => panic!(INV),
	};

	return s.to_string();
}

pub fn undefined_as_encoded_string(e: &TagValue, _: &String) -> String
{
	static ASC: [u8; 8] = [0x41, 0x53, 0x43, 0x49, 0x49, 0, 0, 0];
	static JIS: [u8; 8] = [0x4a, 0x49, 0x53, 0, 0, 0, 0, 0];
	static UNICODE: [u8; 8] = [0x55, 0x4e, 0x49, 0x43, 0x4f, 0x44, 0x45, 0x00];

	match e {
	&TagValue::Undefined(ref v, le) => {
		if v.len() < 8 {
			format!("String w/ truncated preamble {}", numarray_to_string(v))
		} else if v[0..8] == ASC[..] {
			let v8 = &v[8..];
			let s = String::from_utf8_lossy(v8);
			s.into_owned()
		} else if v[0..8] == JIS[..] {
			let v8: Vec<u8> = v[8..].iter().map(|&x| x).collect();
			format!("JIS string {}", numarray_to_string(&v8))
		} else if v[0..8] == UNICODE[..] {
			let v8 = &v[8..];
			// reinterpret as vector of u16
			let v16_size = (v8.len() / 2) as u32;
			let v16 = read_u16_array(le, v16_size, v8);
			String::from_utf16_lossy(&v16)
		} else {
			format!("String w/ undefined encoding {}", numarray_to_string(v))
		}
	},
	_ => panic!(INV),
	}
}

pub fn undefined_as_blob(e: &TagValue, _: &String) -> String
{
	let s = match e {
	&TagValue::Undefined(ref v, _) => {
		format!("Blob of {} bytes", v.len())
	},
	_ => panic!(INV),
	};

	return s.to_string();
}

pub fn apex_tv(e: &TagValue, _: &String) -> String
{
	match e {
		&TagValue::IRational(ref v) => {
			format!("{:.1} Tv APEX", v[0].value())
		},
		_ => panic!(INV),
	}
}

pub fn apex_av(e: &TagValue, _: &String) -> String
{
	match e {
		&TagValue::URational(ref v) => {
			format!("{:.1} Av APEX", v[0].value())
		},
		_ => panic!(INV),
	}
}

pub fn apex_brightness(e: &TagValue, _: &String) -> String
{
	match e {
		&TagValue::IRational(ref v) => {
			// numerator 0xffffffff = unknown
			if v[0].numerator == -1 {
				"Unknown".to_string()
			} else {
				format!("{:.1} APEX", v[0].value())
			}
		},
		_ => panic!(INV),
	}
}

pub fn apex_ev(e: &TagValue, _: &String) -> String
{
	match e {
		&TagValue::IRational(ref v) => {
			// express as fraction, except when zero
			if v[0].numerator == 0 {
				"0 EV APEX".to_string()
			} else {
				format!("{} EV APEX", v[0])
			}
		},
		_ => panic!(INV),
	}
}

pub fn file_source(e: &TagValue, _: &String) -> String
{
	let s = match e {
	&TagValue::Undefined(ref v, _) => {
		if v.len() > 0 && v[0] == 3 {
			"DSC"
		} else {
			"Unknown"
		}
	},
	_ => panic!(INV),
	};

	return s.to_string();
}

pub fn flash_energy(e: &TagValue, _: &String) -> String
{
	match e {
		&TagValue::URational(ref v) => {
			format!("{} BCPS", v[0].value())
		},
		_ => panic!(INV),
	}
}

pub fn metering_mode(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::U16(ref v) => {
			let n = v[0];
			match n {
				0 => "Unknown",
				1 => "Average",
				2 => "Center-weighted average",
				3 => "Spot",
				4 => "Multi-spot",
				5 => "Pattern",
				6 => "Partial",
				255 => "Other",
				_ => return format!("Unknown ({})", n),
			}
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn light_source(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::U16(ref v) => {
			let n = v[0];
			match n {
				0 => "Unknown",
				1 => "Daylight",
				2 => "Fluorescent",
				3 => "Tungsten",
				4 => "Flash",
				9 => "Fine weather",
				10 => "Cloudy weather",
				11 => "Shade",
				12 => "Daylight fluorescent (D)",
				13 => "Day white fluorescent (N)",
				14 => "Cool white fluorescent (W)",
				15 => "White fluorescent (WW)",
				17 => "Standard light A",
				18 => "Standard light B",
				19 => "Standard light C",
				20 => "D55",
				21 => "D65",
				22 => "D75",
				23 => "D50",
				24 => "ISO studio tungsten",
				255 => "Other",
				_ => return format!("Unknown ({})", n),
			}
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn color_space(e: &TagValue, _: &String) -> String
{
	let s = match e {
		&TagValue::U16(ref v) => {
			let n = v[0];
			match n {
				1 => "sRGB",
				65535 => "Uncalibrated",
				_ => return format!("Unknown ({})", n),
			}
		},
		_ => panic!(INV),
	};

	return s.to_string();
}

pub fn flash(e: &TagValue, _: &String) -> String
{
	match e {
		&TagValue::U16(ref v) => {
			let n = v[0];
			let mut b0 = "Did not fire. ";
			let mut b12 = "";
			let mut b34 = "";
			let mut b6 = "";

			if (n & (1 << 5)) > 0 {
				return format!("Does not have a flash.");
			}

			if (n & 1) > 0 {
				b0 = "Fired. ";
				if (n & (1 << 6)) > 0 {
					b6 = "Redeye reduction. "
				} else {
					b6 = "No redeye reduction. "
				}

				// bits 1 and 2
				let m = (n >> 1) & 3;
				if m == 2 {
					b12 = "Strobe ret not detected. ";
				} else if m == 3 {
					b12 = "Strobe ret detected. ";
				}
			}

			// bits 3 and 4
			let m = (n >> 3) & 3;
			if m == 1 {
				b34 = "Forced fire. ";
			} else if m == 2 {
				b34 = "Forced suppresion. ";
			} else if m == 3 {
				b12 = "Auto mode. ";
			}

			format!("{}{}{}{}", b0, b12, b34, b6)
		},
		_ => panic!(INV),
	}
}

pub fn subject_area(e: &TagValue, _: &String) -> String
{
	match e {
		&TagValue::U16(ref v) => {
			match v.len() {
			2 => format!("at pixel {},{}", v[0], v[1]),
			3 => format!("at center {},{} radius {}", v[0], v[1], v[2]),
			4 => format!("at rectangle {},{} width {} height {}", v[0], v[1], v[2], v[3]),
			_ => format!("Unknown ({}) ", numarray_to_string(v)),
			}
		},
		_ => panic!(INV),
	}
}

pub fn subject_location(e: &TagValue, _: &String) -> String
{
	match e {
		&TagValue::U16(ref v) => {
			format!("at pixel {},{}", v[0], v[1])
		},
		_ => panic!(INV),
	}
}

