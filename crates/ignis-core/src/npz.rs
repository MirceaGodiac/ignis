//! `.npz` / `.npy` reading for release tensors (zip of NPY v1–v3; never pickle).
//!
//! Only little-endian (or byte-order-free) numeric/bool dtypes in C order are accepted: that is what
//! `ignis.registry` writes (`np.savez` of contiguous arrays). Anything else is a typed error, never
//! a silent reinterpretation. [`tensor_sha256`] reproduces `ignis.registry._tensor_sha`.

use std::collections::BTreeMap;
use std::io::{Read, Seek};
use std::path::Path;

use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NpzError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("{name}: not an NPY array ({why})")]
    Format { name: String, why: String },
    #[error("{name}: unsupported dtype {descr}")]
    Dtype { name: String, descr: String },
    #[error("{name}: fortran-ordered arrays are not accepted")]
    Fortran { name: String },
}

/// NumPy dtype names as `str(np.dtype)` prints them (the hash includes this string).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dtype {
    F64,
    F32,
    I64,
    I32,
    I16,
    I8,
    U64,
    U32,
    U16,
    U8,
    Bool,
}

impl Dtype {
    fn parse(descr: &str) -> Option<Dtype> {
        let (order, code) = descr.split_at(1);
        if !matches!(order, "<" | "|" | "=") {
            return None;
        }
        Some(match code {
            "f8" => Dtype::F64,
            "f4" => Dtype::F32,
            "i8" => Dtype::I64,
            "i4" => Dtype::I32,
            "i2" => Dtype::I16,
            "i1" => Dtype::I8,
            "u8" => Dtype::U64,
            "u4" => Dtype::U32,
            "u2" => Dtype::U16,
            "u1" => Dtype::U8,
            "b1" => Dtype::Bool,
            _ => return None,
        })
    }
    pub fn numpy_name(self) -> &'static str {
        match self {
            Dtype::F64 => "float64",
            Dtype::F32 => "float32",
            Dtype::I64 => "int64",
            Dtype::I32 => "int32",
            Dtype::I16 => "int16",
            Dtype::I8 => "int8",
            Dtype::U64 => "uint64",
            Dtype::U32 => "uint32",
            Dtype::U16 => "uint16",
            Dtype::U8 => "uint8",
            Dtype::Bool => "bool",
        }
    }
    pub fn size(self) -> usize {
        match self {
            Dtype::F64 | Dtype::I64 | Dtype::U64 => 8,
            Dtype::F32 | Dtype::I32 | Dtype::U32 => 4,
            Dtype::I16 | Dtype::U16 => 2,
            Dtype::I8 | Dtype::U8 | Dtype::Bool => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NpyArray {
    pub dtype: Dtype,
    pub shape: Vec<usize>,
    /// Raw little-endian C-order bytes.
    pub data: Vec<u8>,
}

impl NpyArray {
    pub fn len(&self) -> usize {
        self.shape.iter().product()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Values widened to f64 (row-major).
    pub fn to_f64(&self) -> Vec<f64> {
        let s = self.dtype.size();
        self.data
            .chunks_exact(s)
            .map(|c| match self.dtype {
                Dtype::F64 => f64::from_le_bytes(c.try_into().unwrap()),
                Dtype::F32 => f32::from_le_bytes(c.try_into().unwrap()) as f64,
                Dtype::I64 => i64::from_le_bytes(c.try_into().unwrap()) as f64,
                Dtype::I32 => i32::from_le_bytes(c.try_into().unwrap()) as f64,
                Dtype::I16 => i16::from_le_bytes(c.try_into().unwrap()) as f64,
                Dtype::I8 => c[0] as i8 as f64,
                Dtype::U64 => u64::from_le_bytes(c.try_into().unwrap()) as f64,
                Dtype::U32 => u32::from_le_bytes(c.try_into().unwrap()) as f64,
                Dtype::U16 => u16::from_le_bytes(c.try_into().unwrap()) as f64,
                Dtype::U8 => c[0] as f64,
                Dtype::Bool => (c[0] != 0) as u8 as f64,
            })
            .collect()
    }
}

/// `sha256(f"{dtype}|{','.join(shape)}|" + bytes)` exactly as `ignis.registry._tensor_sha`.
pub fn tensor_sha256(a: &NpyArray) -> String {
    let mut h = Sha256::new();
    let shape: Vec<String> = a.shape.iter().map(|d| d.to_string()).collect();
    h.update(format!("{}|{}|", a.dtype.numpy_name(), shape.join(",")).as_bytes());
    h.update(&a.data);
    hex::encode(h.finalize())
}

pub fn parse_npy(name: &str, bytes: &[u8]) -> Result<NpyArray, NpzError> {
    let fmt = |why: &str| NpzError::Format {
        name: name.to_string(),
        why: why.to_string(),
    };
    if bytes.len() < 10 || &bytes[..6] != b"\x93NUMPY" {
        return Err(fmt("bad magic"));
    }
    let major = bytes[6];
    let (hlen, start) = match major {
        1 => (u16::from_le_bytes([bytes[8], bytes[9]]) as usize, 10),
        2 | 3 => {
            if bytes.len() < 12 {
                return Err(fmt("truncated header"));
            }
            (
                u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize,
                12,
            )
        }
        _ => return Err(fmt("unknown version")),
    };
    let header = bytes
        .get(start..start + hlen)
        .ok_or_else(|| fmt("truncated header"))?;
    let header = std::str::from_utf8(header).map_err(|_| fmt("header not utf-8"))?;
    let descr = dict_value(header, "descr").ok_or_else(|| fmt("no descr"))?;
    let descr = descr
        .trim()
        .trim_matches(|c| c == '\'' || c == '"')
        .to_string();
    let fortran = dict_value(header, "fortran_order").ok_or_else(|| fmt("no fortran_order"))?;
    let shape_src = dict_value(header, "shape").ok_or_else(|| fmt("no shape"))?;
    let dtype = Dtype::parse(&descr).ok_or_else(|| NpzError::Dtype {
        name: name.into(),
        descr: descr.clone(),
    })?;
    let shape: Vec<usize> = shape_src
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<usize>().map_err(|_| fmt("bad shape")))
        .collect::<Result<_, _>>()?;
    let count: usize = shape.iter().product();
    let data = &bytes[start + hlen..];
    if fortran.trim() == "True" && shape.len() > 1 {
        return Err(NpzError::Fortran { name: name.into() });
    }
    if data.len() != count * dtype.size() {
        return Err(fmt("payload size does not match shape"));
    }
    Ok(NpyArray {
        dtype,
        shape,
        data: data.to_vec(),
    })
}

/// Value text of `'key': <value>` in the NPY header dict (values are a quoted string, a bool or a
/// parenthesised tuple; nothing nested beyond that).
fn dict_value<'a>(header: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!("'{key}':");
    let i = header.find(&pat)? + pat.len();
    let rest = header[i..].trim_start();
    let end = if rest.starts_with('(') {
        rest.find(')')? + 1
    } else if rest.starts_with('\'') {
        rest[1..].find('\'')? + 2
    } else {
        rest.find([',', '}'])?
    };
    Some(&rest[..end])
}

pub fn read_npz_from<R: Read + Seek>(reader: R) -> Result<BTreeMap<String, NpyArray>, NpzError> {
    let mut zip = zip::ZipArchive::new(reader)?;
    let mut out = BTreeMap::new();
    for i in 0..zip.len() {
        let mut f = zip.by_index(i)?;
        let name = f.name().to_string();
        let key = name.strip_suffix(".npy").unwrap_or(&name).to_string();
        let mut buf = Vec::with_capacity(f.size() as usize);
        f.read_to_end(&mut buf)?;
        out.insert(key.clone(), parse_npy(&key, &buf)?);
    }
    Ok(out)
}

pub fn read_npz(path: &Path) -> Result<BTreeMap<String, NpyArray>, NpzError> {
    read_npz_from(std::fs::File::open(path)?)
}

impl NpyArray {
    pub fn from_f64(shape: Vec<usize>, values: &[f64]) -> NpyArray {
        assert_eq!(
            shape.iter().product::<usize>(),
            values.len(),
            "shape/len mismatch"
        );
        let mut data = Vec::with_capacity(values.len() * 8);
        for v in values {
            data.extend_from_slice(&v.to_le_bytes());
        }
        NpyArray {
            dtype: Dtype::F64,
            shape,
            data,
        }
    }

    fn descr(&self) -> &'static str {
        match self.dtype {
            Dtype::F64 => "<f8",
            Dtype::F32 => "<f4",
            Dtype::I64 => "<i8",
            Dtype::I32 => "<i4",
            Dtype::I16 => "<i2",
            Dtype::I8 => "|i1",
            Dtype::U64 => "<u8",
            Dtype::U32 => "<u4",
            Dtype::U16 => "<u2",
            Dtype::U8 => "|u1",
            Dtype::Bool => "|b1",
        }
    }

    /// NPY v1.0 bytes (C order), readable by NumPy and by [`parse_npy`].
    pub fn to_npy_bytes(&self) -> Vec<u8> {
        let shape = match self.shape.len() {
            0 => "()".to_string(),
            1 => format!("({},)", self.shape[0]),
            _ => format!(
                "({})",
                self.shape
                    .iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        };
        let mut header = format!(
            "{{'descr': '{}', 'fortran_order': False, 'shape': {}, }}",
            self.descr(),
            shape
        );
        let unpadded = 10 + header.len() + 1;
        header.push_str(&" ".repeat((64 - unpadded % 64) % 64));
        header.push('\n');
        let mut out = Vec::with_capacity(10 + header.len() + self.data.len());
        out.extend_from_slice(b"\x93NUMPY\x01\x00");
        out.extend_from_slice(&(header.len() as u16).to_le_bytes());
        out.extend_from_slice(header.as_bytes());
        out.extend_from_slice(&self.data);
        out
    }
}

/// Write an uncompressed `.npz` (one `<name>.npy` per entry, sorted by name).
pub fn write_npz(path: &Path, arrays: &BTreeMap<String, NpyArray>) -> Result<(), NpzError> {
    use std::io::Write;
    let file = std::fs::File::create(path)
        .map_err(|e| NpzError::Io(std::io::Error::other(e.to_string())))?;
    let mut zw = zip::ZipWriter::new(file);
    let opts =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, a) in arrays {
        zw.start_file(format!("{name}.npy"), opts)
            .map_err(|e| NpzError::Io(std::io::Error::other(e.to_string())))?;
        zw.write_all(&a.to_npy_bytes())
            .map_err(|e| NpzError::Io(std::io::Error::other(e.to_string())))?;
    }
    zw.finish()
        .map_err(|e| NpzError::Io(std::io::Error::other(e.to_string())))?;
    Ok(())
}
