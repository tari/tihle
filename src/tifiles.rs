use num_traits::FromPrimitive;
use std::io::{Error as IoError, Read, Result as IoResult};

use super::checksum::ChecksumRead;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Invalid(&'static str),
    Io(std::io::Error),
}

impl std::convert::From<IoError> for Error {
    fn from(other: IoError) -> Self {
        Error::Io(other)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub comment: Box<[u8]>,
    pub var: Variable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: Box<[u8]>,
    pub ty: VariableType,
    pub version: Option<u8>,
    pub flags: Option<u8>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive)]
pub enum VariableType {
    /// RealObj
    Real = 0,
    /// ListObj
    List = 1,
    /// MatObj
    Matrix = 2,
    /// EquObj
    Equation = 3,
    /// StrngObj
    String = 4,
    /// ProgObj
    Program = 5,
    /// ProtProgObj
    ProtectedProgram = 6,
    /// PictObj
    Picture = 7,
    /// GDBObj
    GraphDatabase = 8,
    // UnknownObj = 9
    // UnknownEquObj = 0xA
    /// NewEquObj
    NewEquation = 0xB,
    /// CplxObj
    Complex = 0xC,
    /// CListObj
    ComplexList = 0xD,
    // UndefObj = 0xE
    /// WindowObj
    Window = 0xF,
    // ZStoObj = 0x10
    // TblRngObj = 0x11
    // LCDObj = 0x12
    /// BackupObj
    Backup = 0x13,
    // AppObj = 0x14 (menus/link only)
    /// AppVarObj
    AppVar = 0x15,
    /// TempProgObj
    TemporaryProgram = 0x16,
    /// GroupObj
    Group = 0x17,
}

impl File {
    pub fn read_from<R: Read>(mut src: R) -> Result<File> {
        let mut buf = [0u8; 11];
        src.read_exact(&mut buf)?;
        if &buf != b"**TI83F*\x1a\x0a\x00" {
            return Err(Error::Invalid("Invalid signature"));
        }

        let mut comment = vec![0u8; 42];
        src.read_exact(&mut comment)?;

        let data_len = read_u16(&mut src)?;

        let mut src = ChecksumRead::from(src);
        let var = Variable::read_from(&mut src)?;
        let ChecksumRead {
            r: mut src,
            sum: actual_sum,
            bytes_read,
        } = src;

        if bytes_read != data_len as usize {
            return Err(Error::Invalid("Data length mismatch"));
        }

        let expected_sum = read_u16(&mut src)?;
        if actual_sum != expected_sum {
            return Err(Error::Invalid("Incorrect checksum"));
        }

        Ok(File {
            comment: comment.into_boxed_slice(),
            var,
        })
    }
}

impl Variable {
    fn read_from<R: Read>(mut src: R) -> Result<Variable> {
        let hdr_len = read_u16(&mut src)?;
        let data_len = read_u16(&mut src)?;
        let ty = match VariableType::from_u8(read_u8(&mut src)?) {
            None => return Err(Error::Invalid("Unrecognized variable type")),
            Some(t) => t,
        };

        let mut raw_name = [0u8; 8];
        src.read_exact(&mut raw_name)?;
        let name_len = raw_name
            .iter()
            .position(|&x| x == 0)
            .unwrap_or(raw_name.len());
        let name: Box<[u8]> = raw_name[..name_len].into();

        let (version, flags) = match hdr_len {
            11 => (None, None),
            13 => (Some(read_u8(&mut src)?), Some(read_u8(&mut src)?)),
            _ => return Err(Error::Invalid("Unexpected variable header length")),
        };

        let data_len_2 = read_u16(&mut src)?;
        if data_len != data_len_2 {
            return Err(Error::Invalid("length/length2 mismatch"));
        }

        let mut data = vec![0u8; data_len as usize];
        src.read_exact(&mut data)?;

        Ok(Variable {
            name,
            ty,
            version,
            flags,
            data,
        })
    }

    /// Get the on-calculator contents of a variable.
    ///
    /// Many types use the first two bytes of data to indicate the size, and those
    /// bytes are included in the variable data here. This function gets only the bytes
    /// that are actual data, not the size bytes.
    pub fn calc_data(&self) -> &[u8] {
        if self.ty != VariableType::Program || self.ty != VariableType::ProtectedProgram {
            unimplemented!("Only programs are currently supported for data");
        }

        &self.data[2..]
    }

    pub fn calc_data_mut(&mut self) -> &mut [u8] {
        if self.ty != VariableType::Program && self.ty != VariableType::ProtectedProgram {
            unimplemented!(
                "Only programs are currently supported for data, not {:?}",
                self.ty
            );
        }

        &mut self.data[2..]
    }

    /// If this variable is an Ion program, patch it to execute as if it were nostub
    /// and return whether it is an Ion program.
    pub fn patch_ion_program(&mut self) -> bool {
        // Ion programs are only even programs, of course.
        if self.ty != VariableType::Program && self.ty != VariableType::ProtectedProgram {
            return false;
        }
        let data = self.calc_data_mut();

        // Ion programs start with the standard tAsmCmp signature (they're never unsquished),
        // then either `ret` or `xor a`, followed by `jr nc, start` and a string description.
        // The `xor a` form is for programs that don't use the Ion libraries, which thus don't
        // need patching.
        if data[..4] != b"\xbb\x6d\xc9\x30"[..] {
            return false;
        }

        // Nop out the ret
        data[2] = 0;
        // Make the jump unconditional
        data[3] = 0x18;
        true
    }
}

fn read_u8<R: Read>(mut src: R) -> IoResult<u8> {
    let mut buf = [0u8; 1];
    src.read_exact(&mut buf[..])?;

    Ok(buf[0])
}

fn read_u16<R: Read>(mut src: R) -> IoResult<u16> {
    let mut buf = [0u8; 2];
    src.read_exact(&mut buf[..])?;

    Ok(u16::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::{File, Variable, VariableType};

    #[test]
    fn read_8xp() {
        let minimal_program = b"**TI83F*\x1a\x0a\x00\
                                File generated by spasm\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                                \x16\0\
                                \x0d\0\
                                \x05\0\
                                \x06\
                                MINIMAL\0\
                                \0\0\
                                \x05\0\
                                \x03\0\
                                \xbb\x6d\xc9\
                                \x18\x04";
        assert_eq!(
            File::read_from(&minimal_program[..]).expect("Should not fail to read file"),
            File {
                comment: b"File generated by spasm\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"[..]
                    .into(),
                var: Variable {
                    name: b"MINIMAL"[..].into(),
                    ty: VariableType::ProtectedProgram,
                    version: Some(0),
                    flags: Some(0),
                    data: vec![3, 0, 0xbb, 0x6d, 0xc9]
                }
            }
        );
    }
}
