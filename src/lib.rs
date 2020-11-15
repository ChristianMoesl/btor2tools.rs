use btor2tools_sys::{
    btor2parser_delete, btor2parser_error, btor2parser_iter_init, btor2parser_iter_next,
    btor2parser_max_id, btor2parser_new, btor2parser_read_lines, fclose, fopen,
    Btor2Line as CBtor2Line, Btor2LineIterator as CBtor2LineIterator, Btor2Parser as CBtor2Parser,
    Btor2SortTag as CBtor2SortTag, Btor2Tag as CBtor2Tag,
};
use std::{
    convert::From,
    ffi::{CStr, CString, NulError},
    fmt,
    marker::PhantomData,
    os::raw::c_char,
    path::Path,
    slice,
};

pub struct Btor2Parser {
    internal: *mut CBtor2Parser,
}

impl Btor2Parser {
    pub fn new() -> Self {
        Self {
            internal: unsafe { btor2parser_new() },
        }
    }

    pub fn read_lines<P>(&mut self, file: P) -> Result<Btor2LineIterator, Btor2ParserError>
    where
        P: AsRef<Path>,
    {
        unsafe {
            let c_file_path = CString::new(file.as_ref().to_str().unwrap())?;
            let c_file_mode = CString::new("r")?;

            let file = fopen(c_file_path.as_ptr(), c_file_mode.as_ptr());

            let result = btor2parser_read_lines(self.internal, file);

            fclose(file);

            if result == 0 {
                Err(Btor2ParserError::new(btor2parser_error(self.internal)))
            } else {
                Ok(Btor2LineIterator::new(self))
            }
        }
    }

    pub fn max_id(&self) -> i64 {
        unsafe { btor2parser_max_id(self.internal) }
    }
}

impl Drop for Btor2Parser {
    fn drop(&mut self) {
        unsafe { btor2parser_delete(self.internal) }
    }
}

#[derive(Debug, Clone)]
pub struct Btor2ParserError {
    details: String,
}

impl Btor2ParserError {
    fn new(str: *const c_char) -> Self {
        unsafe {
            Self {
                details: CStr::from_ptr(str).to_str().unwrap().to_owned(),
            }
        }
    }
}

impl From<NulError> for Btor2ParserError {
    fn from(e: NulError) -> Self {
        Self {
            details: e.to_string(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Btor2LineIterator<'parser> {
    parser: PhantomData<&'parser Btor2Parser>,
    internal: CBtor2LineIterator,
}

impl<'parser> Btor2LineIterator<'parser> {
    fn new(parser: &'parser Btor2Parser) -> Self {
        Self {
            parser: PhantomData,
            internal: unsafe { btor2parser_iter_init(parser.internal) },
        }
    }
}

impl<'parser> Iterator for Btor2LineIterator<'parser> {
    type Item = Btor2Line<'parser>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let c_line = btor2parser_iter_next(&mut self.internal);

            if c_line.is_null() {
                None
            } else {
                Some(Btor2Line::new(c_line))
            }
        }
    }
}

#[derive(Clone)]
pub struct Btor2Line<'parser> {
    parser: PhantomData<&'parser Btor2Parser>,
    internal: *const CBtor2Line,
}

impl<'parser> Btor2Line<'parser> {
    fn new(internal: *mut CBtor2Line) -> Self {
        Self {
            parser: PhantomData,
            internal,
        }
    }

    pub fn id(&self) -> i64 {
        unsafe { (*self.internal).id }
    }

    pub fn lineno(&self) -> i64 {
        unsafe { (*self.internal).lineno }
    }

    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr((*self.internal).name) }
    }

    pub fn tag(&self) -> Btor2Tag {
        unsafe { Btor2Tag::from((*self.internal).tag) }
    }

    pub fn sort(&self) -> Btor2Sort {
        Btor2Sort {
            line: PhantomData,
            internal: self.internal,
        }
    }

    pub fn init(&self) -> i64 {
        unsafe { (*self.internal).init }
    }

    pub fn next(&self) -> i64 {
        unsafe { (*self.internal).next }
    }

    pub fn constant(&self) -> Option<&CStr> {
        wrap_nullable_c_string(unsafe { (*self.internal).constant })
    }

    pub fn symbol(&self) -> Option<&CStr> {
        wrap_nullable_c_string(unsafe { (*self.internal).symbol })
    }

    pub fn args(&self) -> &[i64] {
        unsafe { slice::from_raw_parts((*self.internal).args, (*self.internal).nargs as usize) }
    }
}

impl<'parser> fmt::Debug for Btor2Line<'parser> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Btor2Line")
            .field("id", &self.id())
            .field("lineno", &self.lineno())
            .field("name", &self.name())
            .field("tag", &self.tag())
            .field("sort", &self.sort())
            .field("init", &self.init())
            .field("next", &self.next())
            .field("constant", &self.constant())
            .field("symbol", &self.symbol())
            .field("args", &self.args())
            .finish()
    }
}

#[derive(Copy, Clone)]
pub struct Btor2Sort<'line, 'parser> {
    line: PhantomData<&'line Btor2Line<'parser>>,
    internal: *const CBtor2Line,
}

impl<'line, 'parser> Btor2Sort<'line, 'parser> {
    pub fn id(&self) -> i64 {
        unsafe { (*self.internal).sort.id }
    }

    pub fn tag(&self) -> Btor2SortTag {
        unsafe { Btor2SortTag::from((*self.internal).sort.tag) }
    }

    pub fn name(&self) -> Option<&CStr> {
        wrap_nullable_c_string(unsafe { (*self.internal).sort.name })
    }

    pub fn content(&self) -> Btor2SortContent {
        unsafe {
            match self.tag() {
                Btor2SortTag::Array => Btor2SortContent::Array {
                    index: (*self.internal).sort.__bindgen_anon_1.array.index,
                    element: (*self.internal).sort.__bindgen_anon_1.array.element,
                },
                Btor2SortTag::Bitvec => Btor2SortContent::Bitvec {
                    width: (*self.internal).sort.__bindgen_anon_1.bitvec.width,
                },
            }
        }
    }
}

impl<'line, 'parser> fmt::Debug for Btor2Sort<'line, 'parser> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Btor2Sort")
            .field("id", &self.id())
            .field("tag", &self.tag())
            .field("name", &self.name())
            .field("content", &self.content())
            .finish()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Btor2SortContent {
    Array { index: i64, element: i64 },
    Bitvec { width: u32 },
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum Btor2Tag {
    Add,
    And,
    Bad,
    Concat,
    Const,
    Constraint,
    Constd,
    Consth,
    Dec,
    Eq,
    Fair,
    Iff,
    Implies,
    Inc,
    Init,
    Input,
    Ite,
    Justice,
    Mul,
    Nand,
    Neq,
    Neg,
    Next,
    Nor,
    Not,
    One,
    Ones,
    Or,
    Output,
    Read,
    Redand,
    Redor,
    Redxor,
    Rol,
    Ror,
    Saddo,
    Sdiv,
    Sdivo,
    Sext,
    Sgt,
    Sgte,
    Slice,
    Sll,
    Slt,
    Slte,
    Sort,
    Smod,
    Smulo,
    Sra,
    Srem,
    Srl,
    Ssubo,
    State,
    Sub,
    Uaddo,
    Udiv,
    Uext,
    Ugt,
    Ugte,
    Ult,
    Ulte,
    Umulo,
    Urem,
    Usubo,
    Write,
    Xnor,
    Xor,
    Zero,
}

impl From<CBtor2Tag> for Btor2Tag {
    fn from(raw: CBtor2Tag) -> Btor2Tag {
        unsafe { core::mem::transmute(raw) }
    }
}

impl Into<CBtor2Tag> for Btor2Tag {
    fn into(self) -> CBtor2Tag {
        unsafe { core::mem::transmute(self) }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum Btor2SortTag {
    Array,
    Bitvec,
}

impl From<CBtor2SortTag> for Btor2SortTag {
    fn from(raw: CBtor2SortTag) -> Btor2SortTag {
        unsafe { std::mem::transmute(raw) }
    }
}

impl Into<CBtor2SortTag> for Btor2SortTag {
    fn into(self) -> CBtor2SortTag {
        unsafe { std::mem::transmute(self) }
    }
}

fn wrap_nullable_c_string<'a>(str: *const c_char) -> Option<&'a CStr> {
    unsafe {
        if str.is_null() {
            None
        } else {
            Some(CStr::from_ptr(str))
        }
    }
}
